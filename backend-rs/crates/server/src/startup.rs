use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use crate::{
    configuration::{AuthSettings, CorsSettings, DatabaseSettings, MqttSettings, Settings},
    http::{AppState, auth::AuthLayer, router},
    infra::{
        self,
        keycloak::{AuthStack, JwksProvider},
        pg_cluster::PgTreeClusterRepository,
        pg_evaluation::PgEvaluationRepository,
        pg_region::PgRegionRepository,
        pg_sensor::PgSensorRepository,
        pg_tree::PgTreeRepository,
        pg_vehicle::PgVehicleRepository,
        pg_watering_plan::PgWateringPlanRepository,
        system_info::DefaultSystemInfoProvider,
    },
    service::{
        cluster_service::ClusterService,
        evaluation_service::EvaluationService,
        event_bus::{EventBus, InMemoryEventBus},
        handlers::cluster_recalc::ClusterRecalculationHandler,
        handlers::cluster_status::ClusterStatusAggregatorHandler,
        handlers::tree_watering::TreeWateringFromSensorHandler,
        region_service::RegionService,
        sensor_service::SensorService,
        tree_service::TreeService,
        vehicle_service::VehicleService,
        watering_execution_service::WateringExecutionService,
        watering_plan_service::WateringPlanService,
    },
};

pub struct Application {
    port: u16,
    listener: TcpListener,
    state: Arc<AppState>,
    base_url: String,
    cors: CorsSettings,
    auth_layer: AuthLayer,
    _jwks: Arc<JwksProvider>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&config.database)
            .await
            .expect("failed to connect to database");

        let address = format!("{}:{}", config.application.host, config.application.port);
        let app = Self::build_with_pool(
            pool,
            &address,
            config.application.base_url.clone(),
            config.cors.clone(),
            config.auth.clone(),
        )
        .await?;

        spawn_background_tasks(&config, &app);
        Ok(app)
    }

    /// Variant that takes mqtt settings explicitly. Used by tests that
    /// construct `Application` from a pool but want to drive (or skip) the
    /// MQTT subscriber independently.
    #[allow(dead_code)]
    pub async fn build_with_mqtt(
        pool: PgPool,
        address: &str,
        base_url: String,
        cors: CorsSettings,
        auth: AuthSettings,
        mqtt: MqttSettings,
    ) -> Result<Self, std::io::Error> {
        let app = Self::build_with_pool(pool, address, base_url, cors, auth).await?;
        if let Err(e) = infra::mqtt::spawn(mqtt, app.state.sensor_service.clone()) {
            tracing::error!(error = %e, "mqtt subscriber not started");
        }
        Ok(app)
    }

    pub async fn build_with_pool(
        pool: PgPool,
        address: &str,
        base_url: String,
        cors: CorsSettings,
        auth: AuthSettings,
    ) -> Result<Self, std::io::Error> {
        // Repositories
        let region_repo = Arc::new(PgRegionRepository::new(pool.clone()));
        let region_reader: Arc<dyn crate::domain::region::RegionReader> = region_repo.clone();
        let region_writer: Arc<dyn crate::domain::region::RegionWriter> = region_repo;
        let tree_repo = Arc::new(PgTreeRepository::new(pool.clone()));
        let tree_reader: Arc<dyn crate::domain::tree::TreeReader> = tree_repo.clone();
        let tree_writer: Arc<dyn crate::domain::tree::TreeWriter> = tree_repo;
        let sensor_repo = Arc::new(PgSensorRepository::new(pool.clone()));
        let sensor_reader: Arc<dyn crate::domain::sensor::SensorReader> = sensor_repo.clone();
        let sensor_writer: Arc<dyn crate::domain::sensor::SensorWriter> = sensor_repo.clone();
        let sensor_reading_reader: Arc<dyn crate::domain::sensor::SensorReadingReader> =
            sensor_repo.clone();
        let sensor_reading_writer: Arc<dyn crate::domain::sensor::SensorReadingWriter> =
            sensor_repo;
        let vehicle_repo = Arc::new(PgVehicleRepository::new(pool.clone()));
        let vehicle_reader: Arc<dyn crate::domain::vehicle::VehicleReader> = vehicle_repo.clone();
        let vehicle_writer: Arc<dyn crate::domain::vehicle::VehicleWriter> = vehicle_repo;
        let cluster_repo = Arc::new(PgTreeClusterRepository::new(pool.clone()));
        let cluster_reader: Arc<dyn crate::domain::cluster::TreeClusterReader> =
            cluster_repo.clone();
        let cluster_writer: Arc<dyn crate::domain::cluster::TreeClusterWriter> = cluster_repo;
        let watering_plan_repo = Arc::new(PgWateringPlanRepository::new(pool.clone()));
        let watering_plan_reader: Arc<dyn crate::domain::watering_plan::WateringPlanReader> =
            watering_plan_repo.clone();
        let watering_plan_writer: Arc<dyn crate::domain::watering_plan::WateringPlanWriter> =
            watering_plan_repo;
        let evaluation_repo: Arc<dyn crate::domain::evaluation::EvaluationRepository> =
            Arc::new(PgEvaluationRepository::new(pool));

        let AuthStack {
            auth_service,
            user_service,
            auth_layer,
            jwks,
        } = infra::keycloak::build(&auth).await?;

        // Event handlers
        let cluster_recalc_handler = Arc::new(ClusterRecalculationHandler::new(
            cluster_reader.clone(),
            cluster_writer.clone(),
            tree_reader.clone(),
            region_reader.clone(),
        ));
        let cluster_status_handler = Arc::new(ClusterStatusAggregatorHandler::new(
            cluster_reader.clone(),
            cluster_writer.clone(),
            tree_reader.clone(),
        ));
        let tree_watering_handler = Arc::new(TreeWateringFromSensorHandler::new(
            tree_reader.clone(),
            tree_writer.clone(),
        ));

        // Event bus
        let handlers: Vec<Arc<dyn crate::service::event_bus::EventHandler>> = vec![
            cluster_recalc_handler as Arc<dyn crate::service::event_bus::EventHandler>,
            cluster_status_handler as Arc<dyn crate::service::event_bus::EventHandler>,
            tree_watering_handler as Arc<dyn crate::service::event_bus::EventHandler>,
        ];
        let event_bus: Arc<dyn EventBus> = Arc::new(InMemoryEventBus::new(handlers));

        // Domain services
        let region_service = Arc::new(RegionService::new(region_reader, region_writer));
        let tree_service = Arc::new(TreeService::new(
            tree_reader.clone(),
            tree_writer.clone(),
            event_bus.clone(),
        ));
        let sensor_service = Arc::new(SensorService::new(
            sensor_reader,
            sensor_writer,
            sensor_reading_reader,
            sensor_reading_writer,
            tree_reader.clone(),
            tree_writer.clone(),
            event_bus.clone(),
        ));
        let vehicle_service = Arc::new(VehicleService::new(vehicle_reader, vehicle_writer));
        let cluster_service = Arc::new(ClusterService::new(
            cluster_reader,
            cluster_writer,
            tree_reader.clone(),
            tree_writer,
            event_bus.clone(),
        ));
        let watering_plan_service = Arc::new(WateringPlanService::new(
            watering_plan_reader.clone(),
            watering_plan_writer.clone(),
            event_bus.clone(),
        ));
        let watering_execution_service = Arc::new(WateringExecutionService::new(
            watering_plan_reader,
            watering_plan_writer,
            event_bus.clone(),
        ));
        let evaluation_service = Arc::new(EvaluationService::new(evaluation_repo));
        let info_provider: Arc<dyn crate::domain::info::SystemInfoProvider> =
            Arc::new(DefaultSystemInfoProvider::new());

        let state = Arc::new(AppState {
            region_service,
            tree_service,
            sensor_service,
            vehicle_service,
            cluster_service,
            watering_plan_service,
            watering_execution_service,
            evaluation_service,
            auth_service,
            user_service,
            info_provider,
        });

        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();

        Ok(Self {
            port,
            listener,
            state,
            base_url,
            cors,
            auth_layer,
            _jwks: jwks,
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

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let app = router(self.state, &self.base_url, &self.cors, self.auth_layer);
        tracing::info!("listening on {}", self.listener.local_addr()?);
        axum::serve(self.listener, app).await
    }
}

pub async fn get_connection_pool(config: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(config.connection_options())
        .await
}

/// Starts every background task that has its `enabled` flag set in
/// `config`. Failures are logged and the HTTP server still comes up — a
/// missing broker should not bring down a running deployment.
fn spawn_background_tasks(config: &Settings, app: &Application) {
    if let Err(e) = infra::mqtt::spawn(config.mqtt.clone(), app.state.sensor_service.clone()) {
        tracing::error!(error = %e, "mqtt subscriber not started");
    }
}
