use std::sync::Arc;
use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use crate::{
    configuration::{CorsSettings, DatabaseSettings, Settings},
    http::{AppState, FeatureFlags, NearestTreeLimits, auth::AuthLayer, router},
    infra::{
        self,
        health::{
            DefaultReadiness, HealthProbe, feature_probe::FeatureProbe,
            keycloak_probe::KeycloakProbe, mqtt_probe::MqttProbe, pg_probe::PgProbe,
            spawn as spawn_health,
        },
        keycloak::{AuthStack, JwksProvider},
        mqtt::MqttHealthState,
        pg_cluster::PgTreeClusterRepository,
        pg_evaluation::PgEvaluationRepository,
        pg_organization::PgOrganizationRepository,
        pg_region::PgRegionRepository,
        pg_role::PgRoleRepository,
        pg_sensor::PgSensorRepository,
        pg_sensor_model::PgSensorModelRepository,
        pg_start_point::PgStartPointRepository,
        pg_tree::PgTreeRepository,
        pg_vehicle::PgVehicleRepository,
        pg_watering_plan::PgWateringPlanRepository,
        statistics_repo::PgStatisticsRepo,
        system_info::DefaultSystemInfoProvider,
        update_checker::UpdateChecker,
    },
    service::{
        authorization::AuthorizationService,
        cluster_service::ClusterService,
        evaluation_service::EvaluationService,
        event_bus::{EventBus, EventHandler, InMemoryEventBus},
        handlers::cluster_recalc::ClusterRecalculationHandler,
        handlers::cluster_soil_recalc::ClusterSoilRecalcHandler,
        handlers::cluster_status::ClusterStatusAggregatorHandler,
        handlers::tree_watering::TreeWateringFromSensorHandler,
        organization_service::OrganizationService,
        region_service::RegionService,
        role_service::RoleService,
        sensor_service::SensorService,
        start_point_service::StartPointService,
        tree_service::TreeService,
        user_service::UserService,
        vehicle_service::VehicleService,
        watering_execution_service::WateringExecutionService,
        watering_plan_service::WateringPlanService,
    },
};
use domain::info::{
    HealthSnapshotReader, ReadinessReader, ServiceName, StatisticsReader, SystemInfoProvider,
};

pub struct Application {
    port: u16,
    listener: TcpListener,
    state: Arc<AppState>,
    base_url: url::Url,
    cors: CorsSettings,
    auth_layer: AuthLayer,
    _jwks: Arc<JwksProvider>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    mqtt_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&config.database)
            .await
            .expect("failed to connect to database");
        let address = format!("{}:{}", config.application.host, config.application.port);
        Self::build_with_pool(pool, &address, config).await
    }

    pub async fn build_with_pool(
        pool: PgPool,
        address: &str,
        settings: Settings,
    ) -> Result<Self, std::io::Error> {
        let AuthStack {
            user_repo,
            auth_layer,
            jwks,
        } = infra::keycloak::build(&settings.auth).await?;
        let token_validator = auth_layer.validator.clone();

        let probe_http_client =
            build_http_client(Duration::from_secs(settings.info.health_probe_timeout_secs));
        let update_checker = Arc::new(UpdateChecker::new(
            env!("CARGO_PKG_VERSION").to_string(),
            settings.info.update_check_repo.clone(),
        ));
        let info_provider: Arc<dyn SystemInfoProvider> = Arc::new(DefaultSystemInfoProvider::new(
            &settings,
            update_checker.clone(),
        ));

        let sensor_offline_after = chrono::Duration::seconds(
            i64::try_from(settings.sensor.offline_after_secs).unwrap_or(i64::MAX),
        );
        let repos = Repositories::build(&pool, sensor_offline_after);
        let profile_repo = Arc::new(infra::pg_user_profile::PgUserProfileRepository::new(
            pool.clone(),
        ));
        let user_service = Arc::new(UserService::new(
            user_repo.clone(),
            profile_repo.clone(),
            profile_repo.clone(),
            repos.role_reader.clone(),
            repos.role_writer.clone(),
            repos.organization_reader.clone(),
            settings.auth.enabled,
        ));
        let event_bus = build_event_bus(&repos);
        let route_optimizer: Option<Arc<dyn domain::routing::RouteOptimizer>> =
            settings.routing.enabled.then(|| {
                Arc::new(infra::streamlet::StreamletRouteOptimizer::new(
                    &settings.routing,
                )) as Arc<dyn domain::routing::RouteOptimizer>
            });
        let services = Services::build(
            &repos,
            event_bus,
            route_optimizer,
            settings.routing.tree_demand_liters,
            repos.start_point_reader.clone(),
            repos.start_point_writer.clone(),
            profile_repo.clone(),
            profile_repo,
            settings.auth.enabled,
        );

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let (mqtt_state, mqtt_handle) =
            spawn_mqtt_subscriber(&settings, services.sensor.clone(), shutdown_rx);
        let (health_reader, readiness_reader) =
            spawn_health_probes(&pool, &settings, probe_http_client.clone(), mqtt_state).await;
        let _update_handle = infra::update_checker::spawn(
            update_checker,
            probe_http_client,
            Duration::from_secs(settings.info.update_check_interval_secs),
        );

        let state = Arc::new(AppState {
            region_service: services.region,
            tree_service: services.tree,
            sensor_service: services.sensor,
            vehicle_service: services.vehicle,
            cluster_service: services.cluster,
            watering_plan_service: services.watering_plan,
            watering_execution_service: services.watering_execution,
            evaluation_service: services.evaluation,
            user_service,
            info_provider,
            health_reader,
            readiness_reader,
            statistics_reader: repos.statistics,
            token_validator,
            feature_flags: FeatureFlags {
                routing_enabled: settings.routing.enabled,
                plugins_enabled: settings.plugins.enabled,
            },
            nearest_tree_limits: NearestTreeLimits {
                max_radius_meters: settings.map.nearest_tree_max_radius,
                default_limit: settings.map.nearest_tree_default_limit,
                max_limit: settings.map.nearest_tree_max_limit,
            },
            frontend_config_js: crate::http::render_frontend_config_js(&settings.auth).into(),
            start_point_service: services.start_point,
            organization_service: services.organization,
            role_service: services.role,
            authorization_service: services.authorization,
        });

        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            port,
            listener,
            state,
            base_url: settings.application.base_url,
            cors: settings.cors,
            auth_layer,
            _jwks: jwks,
            shutdown_tx,
            mqtt_handle,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Shared application state — exposed so background tasks (MQTT
    /// ingestor, scheduled jobs) and integration tests can call services
    /// directly without going through HTTP.
    pub fn state(&self) -> Arc<AppState> {
        self.state.clone()
    }

    /// Serves HTTP until SIGTERM / ctrl-c, then drains in-flight requests,
    /// signals background tasks to stop, and waits (bounded) for the MQTT
    /// subscriber so an in-flight ingest is not cut off mid-write.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let app = router(
            self.state,
            self.base_url.as_str(),
            &self.cors,
            self.auth_layer,
        );
        tracing::info!("listening on {}", self.listener.local_addr()?);
        axum::serve(self.listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        let _ = self.shutdown_tx.send(true);
        if let Some(handle) = self.mqtt_handle
            && tokio::time::timeout(Duration::from_secs(10), handle)
                .await
                .is_err()
        {
            tracing::warn!("mqtt subscriber did not stop within 10s; exiting anyway");
        }
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("ctrl-c handler must install");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler must install")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received; draining in-flight requests");
}

pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(config.connection_options())
        .await
}

struct Repositories {
    organization_reader: Arc<dyn domain::organization::OrganizationReader>,
    organization_writer: Arc<dyn domain::organization::OrganizationWriter>,
    role_reader: Arc<dyn domain::role::RoleReader>,
    role_writer: Arc<dyn domain::role::RoleWriter>,
    region_reader: Arc<dyn domain::region::RegionReader>,
    region_writer: Arc<dyn domain::region::RegionWriter>,
    tree_reader: Arc<dyn domain::tree::TreeReader>,
    tree_writer: Arc<dyn domain::tree::TreeWriter>,
    sensor_reader: Arc<dyn domain::sensor::SensorReader>,
    sensor_writer: Arc<dyn domain::sensor::SensorWriter>,
    sensor_reading_reader: Arc<dyn domain::sensor::SensorReadingReader>,
    sensor_reading_writer: Arc<dyn domain::sensor::SensorReadingWriter>,
    sensor_model_reader: Arc<dyn domain::sensor_model::SensorModelReader>,
    vehicle_reader: Arc<dyn domain::vehicle::VehicleReader>,
    vehicle_writer: Arc<dyn domain::vehicle::VehicleWriter>,
    cluster_reader: Arc<dyn domain::cluster::TreeClusterReader>,
    cluster_writer: Arc<dyn domain::cluster::TreeClusterWriter>,
    watering_plan_reader: Arc<dyn domain::watering_plan::WateringPlanReader>,
    watering_plan_writer: Arc<dyn domain::watering_plan::WateringPlanWriter>,
    evaluation: Arc<dyn domain::evaluation::EvaluationRepository>,
    statistics: Arc<dyn StatisticsReader>,
    start_point_reader: Arc<dyn domain::start_point::StartPointReader>,
    start_point_writer: Arc<dyn domain::start_point::StartPointWriter>,
}

impl Repositories {
    fn build(pool: &PgPool, sensor_offline_after: chrono::Duration) -> Self {
        let organization_repo = Arc::new(PgOrganizationRepository::new(pool.clone()));
        let role_repo = Arc::new(PgRoleRepository::new(pool.clone()));
        let region_repo = Arc::new(PgRegionRepository::new(pool.clone()));
        let tree_repo = Arc::new(PgTreeRepository::new(pool.clone()));
        let sensor_repo = Arc::new(PgSensorRepository::new(pool.clone(), sensor_offline_after));
        let vehicle_repo = Arc::new(PgVehicleRepository::new(pool.clone()));
        let cluster_repo = Arc::new(PgTreeClusterRepository::new(pool.clone()));
        let watering_plan_repo = Arc::new(PgWateringPlanRepository::new(pool.clone()));
        let start_point_repo = Arc::new(PgStartPointRepository::new(pool.clone()));

        Self {
            organization_reader: organization_repo.clone(),
            organization_writer: organization_repo,
            role_reader: role_repo.clone(),
            role_writer: role_repo,
            region_reader: region_repo.clone(),
            region_writer: region_repo,
            tree_reader: tree_repo.clone(),
            tree_writer: tree_repo,
            sensor_reader: sensor_repo.clone(),
            sensor_writer: sensor_repo.clone(),
            sensor_reading_reader: sensor_repo.clone(),
            sensor_reading_writer: sensor_repo,
            sensor_model_reader: Arc::new(PgSensorModelRepository::new(pool.clone())),
            vehicle_reader: vehicle_repo.clone(),
            vehicle_writer: vehicle_repo,
            cluster_reader: cluster_repo.clone(),
            cluster_writer: cluster_repo,
            watering_plan_reader: watering_plan_repo.clone(),
            watering_plan_writer: watering_plan_repo,
            evaluation: Arc::new(PgEvaluationRepository::new(pool.clone())),
            statistics: Arc::new(PgStatisticsRepo::new(pool.clone())),
            start_point_reader: start_point_repo.clone(),
            start_point_writer: start_point_repo,
        }
    }
}

struct Services {
    region: Arc<RegionService>,
    tree: Arc<TreeService>,
    sensor: Arc<SensorService>,
    vehicle: Arc<VehicleService>,
    cluster: Arc<ClusterService>,
    watering_plan: Arc<WateringPlanService>,
    watering_execution: Arc<WateringExecutionService>,
    evaluation: Arc<EvaluationService>,
    start_point: Arc<StartPointService>,
    organization: Arc<OrganizationService>,
    role: Arc<RoleService>,
    authorization: Arc<AuthorizationService>,
}

impl Services {
    #[allow(clippy::too_many_arguments)]
    fn build(
        repos: &Repositories,
        event_bus: Arc<dyn EventBus>,
        route_optimizer: Option<Arc<dyn domain::routing::RouteOptimizer>>,
        tree_demand_liters: f64,
        start_point_reader: Arc<dyn domain::start_point::StartPointReader>,
        start_point_writer: Arc<dyn domain::start_point::StartPointWriter>,
        profile_reader: Arc<dyn domain::user::UserProfileReader>,
        profile_writer: Arc<dyn domain::user::UserProfileWriter>,
        auth_enabled: bool,
    ) -> Self {
        Self {
            region: Arc::new(RegionService::new(
                repos.region_reader.clone(),
                repos.region_writer.clone(),
            )),
            tree: Arc::new(TreeService::new(
                repos.tree_reader.clone(),
                repos.tree_writer.clone(),
                event_bus.clone(),
            )),
            sensor: Arc::new(SensorService::new(
                repos.sensor_reader.clone(),
                repos.sensor_writer.clone(),
                repos.sensor_reading_reader.clone(),
                repos.sensor_reading_writer.clone(),
                repos.sensor_model_reader.clone(),
                repos.tree_reader.clone(),
                repos.tree_writer.clone(),
                repos.cluster_reader.clone(),
                event_bus.clone(),
            )),
            vehicle: Arc::new(VehicleService::new(
                repos.vehicle_reader.clone(),
                repos.vehicle_writer.clone(),
            )),
            cluster: Arc::new(ClusterService::new(
                repos.cluster_reader.clone(),
                repos.cluster_writer.clone(),
                repos.tree_reader.clone(),
                repos.tree_writer.clone(),
                event_bus.clone(),
            )),
            watering_plan: Arc::new(WateringPlanService::new(
                repos.watering_plan_reader.clone(),
                repos.watering_plan_writer.clone(),
                repos.cluster_reader.clone(),
                repos.vehicle_reader.clone(),
                event_bus.clone(),
                route_optimizer,
                tree_demand_liters,
                start_point_reader.clone(),
            )),
            watering_execution: Arc::new(WateringExecutionService::new(
                repos.watering_plan_reader.clone(),
                repos.watering_plan_writer.clone(),
                event_bus.clone(),
            )),
            evaluation: Arc::new(EvaluationService::new(repos.evaluation.clone())),
            start_point: Arc::new(StartPointService::new(
                start_point_reader.clone(),
                start_point_writer,
            )),
            organization: Arc::new(OrganizationService::new(
                repos.organization_reader.clone(),
                repos.organization_writer.clone(),
                repos.role_reader.clone(),
                repos.role_writer.clone(),
                profile_reader,
                event_bus.clone(),
            )),
            role: Arc::new(RoleService::new(
                repos.role_reader.clone(),
                repos.role_writer.clone(),
                profile_writer,
                event_bus,
            )),
            authorization: Arc::new(AuthorizationService::new(
                repos.organization_reader.clone(),
                repos.role_reader.clone(),
                auth_enabled,
            )),
        }
    }
}

fn build_http_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("reqwest client must build")
}

fn build_event_bus(repos: &Repositories) -> Arc<dyn EventBus> {
    let handlers: Vec<Arc<dyn EventHandler>> = vec![
        Arc::new(ClusterRecalculationHandler::new(
            repos.cluster_reader.clone(),
            repos.cluster_writer.clone(),
            repos.tree_reader.clone(),
            repos.region_reader.clone(),
        )),
        Arc::new(ClusterSoilRecalcHandler::new(
            repos.tree_reader.clone(),
            repos.tree_writer.clone(),
            repos.cluster_reader.clone(),
            repos.sensor_reading_reader.clone(),
        )),
        Arc::new(ClusterStatusAggregatorHandler::new(
            repos.cluster_reader.clone(),
            repos.cluster_writer.clone(),
            repos.tree_reader.clone(),
        )),
        Arc::new(TreeWateringFromSensorHandler::new(
            repos.tree_reader.clone(),
            repos.tree_writer.clone(),
            repos.cluster_reader.clone(),
            repos.sensor_reading_reader.clone(),
        )),
    ];
    Arc::new(InMemoryEventBus::new(handlers))
}

fn spawn_mqtt_subscriber(
    settings: &Settings,
    sensor_service: Arc<SensorService>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> (Arc<MqttHealthState>, Option<tokio::task::JoinHandle<()>>) {
    match infra::mqtt::spawn(settings.mqtt.clone(), sensor_service, shutdown) {
        Ok((state, handle)) => (state, handle),
        Err(e) => {
            tracing::error!(error = %e, "mqtt subscriber not started");
            (Arc::new(MqttHealthState::default()), None)
        }
    }
}

async fn spawn_health_probes(
    pool: &PgPool,
    settings: &Settings,
    probe_http_client: reqwest::Client,
    mqtt_state: Arc<MqttHealthState>,
) -> (Arc<dyn HealthSnapshotReader>, Arc<dyn ReadinessReader>) {
    let pg_probe: Arc<dyn HealthProbe> = Arc::new(PgProbe::new(pool.clone()));
    let readiness: Arc<dyn ReadinessReader> =
        Arc::new(DefaultReadiness::new(vec![pg_probe.clone()]));

    let probes: Vec<Arc<dyn HealthProbe>> = vec![
        pg_probe,
        Arc::new(KeycloakProbe::new(
            settings.auth.enabled,
            Some(&settings.auth.issuer_url),
            probe_http_client,
            Duration::from_secs(settings.info.health_probe_timeout_secs),
        )),
        Arc::new(MqttProbe::new(settings.mqtt.enabled, mqtt_state)),
        Arc::new(FeatureProbe::new(
            ServiceName::Routing,
            settings.routing.enabled,
        )),
        Arc::new(FeatureProbe::new(
            ServiceName::Plugins,
            settings.plugins.enabled,
        )),
    ];
    let (coordinator, _handle) = spawn_health(
        probes,
        Duration::from_secs(settings.info.health_check_interval_secs),
    )
    .await;
    (coordinator, readiness)
}
