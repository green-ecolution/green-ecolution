use std::sync::Arc;
use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use domain::authorization::ResourceCatalog;
use domain::info::{
    HealthSnapshotReader, ReadinessReader, ServiceName, StatisticsReader, SystemInfoProvider,
};
use server::{
    configuration::{CorsSettings, DatabaseSettings, Settings},
    http::{
        AppOrigins, AppState, FeatureFlags, NearestTreeLimits, OidcSwaggerSettings,
        auth::AuthLayer, router,
    },
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
        pg_comment::PgCommentRepository,
        pg_evaluation::PgEvaluationRepository,
        pg_operation_plan::PgOperationPlanRepository,
        pg_organization::PgOrganizationRepository,
        pg_plugin::PgPluginRepository,
        pg_region::PgRegionRepository,
        pg_role::PgRoleRepository,
        pg_sensor::PgSensorRepository,
        pg_sensor_model::PgSensorModelRepository,
        pg_settings::PgSettingsRepository,
        pg_start_point::PgStartPointRepository,
        pg_tree::PgTreeRepository,
        pg_vehicle::PgVehicleRepository,
        statistics_repo::PgStatisticsRepo,
        system_info::DefaultSystemInfoProvider,
        update_checker::UpdateChecker,
    },
    irrigation::{
        IrrigationConfig,
        settings::{IrrigationDefaults, repository::PgIrrigationSettings},
    },
    module::Module,
    service::{
        authorization::AuthorizationService,
        cluster_service::ClusterService,
        comment_service::CommentService,
        evaluation_service::EvaluationService,
        event_bus::{EventBus, EventHandler, InMemoryEventBus},
        handlers::cluster_just_watered::ClusterJustWateredHandler,
        handlers::cluster_recalc::ClusterRecalculationHandler,
        handlers::cluster_soil_recalc::ClusterSoilRecalcHandler,
        handlers::cluster_status::ClusterStatusAggregatorHandler,
        handlers::tree_watering::TreeWateringFromSensorHandler,
        operation_execution_service::OperationExecutionService,
        operation_plan_service::OperationPlanService,
        organization_service::OrganizationService,
        planning::PlanningRegistry,
        plugin_ingest_service::PluginIngestService,
        plugin_proxy_policy::ProxyPolicy,
        plugin_proxy_service::PluginProxyService,
        plugin_sensor_ingest_service::PluginSensorIngestService,
        plugin_service::PluginService,
        plugin_view_ticket_service::PluginViewTicketService,
        region_service::RegionService,
        role_service::RoleService,
        sensor_service::SensorService,
        start_point_service::StartPointService,
        tree_service::TreeService,
        user_service::UserService,
        vehicle_service::VehicleService,
    },
};

pub struct Application {
    port: u16,
    listener: TcpListener,
    state: Arc<AppState>,
    base_url: url::Url,
    cors: CorsSettings,
    auth_layer: AuthLayer,
    oidc_swagger: OidcSwaggerSettings,
    request_timeout: Duration,
    _jwks: Arc<JwksProvider>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    mqtt_handle: Option<tokio::task::JoinHandle<()>>,
    module_routes: Vec<server::module::ModuleRoutes>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&config.database)
            .await
            .expect("failed to connect to database");
        let address = format!("{}:{}", config.application.host, config.application.port);
        Self::build_with_modules(pool, &address, config, crate::modules::registered()).await
    }

    pub async fn build_with_pool(
        pool: PgPool,
        address: &str,
        settings: Settings,
    ) -> Result<Self, std::io::Error> {
        Self::build_with_modules(pool, address, settings, crate::modules::registered()).await
    }

    pub async fn build_with_modules(
        pool: PgPool,
        address: &str,
        settings: Settings,
        modules: Vec<Box<dyn Module>>,
    ) -> Result<Self, std::io::Error> {
        server::module::check_modules(&server::http::v1::core_openapi(), &modules).map_err(
            |err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()),
        )?;
        let resource_catalog = Arc::new(
            ResourceCatalog::new(modules.iter().flat_map(|m| m.resources().iter().copied()))
                .map_err(|e| std::io::Error::other(format!("invalid resource catalog: {e}")))?,
        );
        for configured in settings.modules.keys() {
            if !modules.iter().any(|m| m.key().as_str() == configured) {
                tracing::warn!(module = %configured, "modules.{configured} is configured but no such module is registered");
            }
        }
        let config = Arc::new(settings.clone());
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
        let repos = Repositories::build(
            &pool,
            sensor_offline_after,
            settings.sensor.defect_streak,
            &resource_catalog,
        );
        let defaults = instance_defaults(&settings)?;
        let settings_repo = Arc::new(PgSettingsRepository::new(
            pool.clone(),
            defaults,
            repos.organization_reader.clone(),
        ));
        let irrigation_config = IrrigationConfig::from_settings(&settings).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid configuration: {err}"),
            )
        })?;
        let irrigation_settings = Arc::new(PgIrrigationSettings::new(
            pool.clone(),
            IrrigationDefaults::from_config(&irrigation_config).map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("invalid configuration: {err}"),
                )
            })?,
            repos.organization_reader.clone(),
        ));
        let profile_repo = Arc::new(infra::pg_user_profile::PgUserProfileRepository::new(
            pool.clone(),
        ));
        let settings_service = Arc::new(server::service::settings_service::SettingsService::new(
            settings_repo.clone(),
            settings_repo.clone(),
            settings_repo.clone(),
            profile_repo.clone(),
            defaults,
        ));
        let activation_defaults: std::collections::BTreeMap<_, _> = modules
            .iter()
            .map(|module| {
                let key = module.key();
                let default = settings
                    .modules
                    .get(key.as_str())
                    .and_then(|m| m.default_enabled)
                    .unwrap_or(module.default_enabled());
                (key, default)
            })
            .collect();
        let module_activation_repo = Arc::new(
            server::infra::pg_module_activation::PgModuleActivationRepository::new(
                pool.clone(),
                repos.organization_reader.clone(),
                activation_defaults.clone(),
            ),
        );
        let module_activation_service = Arc::new(
            server::service::module_activation_service::ModuleActivationService::new(
                module_activation_repo.clone(),
                module_activation_repo.clone(),
                repos.organization_reader.clone(),
                activation_defaults.into_keys().collect(),
            ),
        );
        let user_service = Arc::new(UserService::new(
            user_repo.clone(),
            profile_repo.clone(),
            profile_repo.clone(),
            repos.role_reader.clone(),
            repos.role_writer.clone(),
            repos.organization_reader.clone(),
            repos.organization_writer.clone(),
            settings.auth.enabled,
        ));
        let event_bus = Arc::new(server::service::event_bus::DeferredEventBus::new());
        let route_optimizer: Option<Arc<dyn domain::routing::RouteOptimizer>> =
            settings.routing.enabled.then(|| {
                Arc::new(infra::streamlet::StreamletRouteOptimizer::new(
                    &settings.routing,
                )) as Arc<dyn domain::routing::RouteOptimizer>
            });
        let targets = Arc::new(server::service::targets::TargetRegistry::new());
        let planning = Arc::new(PlanningRegistry::new());
        let statistics = Arc::new(server::service::statistics_registry::StatisticsRegistry::new());
        let services = Services::build(
            &repos,
            event_bus.clone(),
            targets.clone(),
            planning.clone(),
            module_activation_repo.clone(),
            route_optimizer,
            repos.start_point_reader.clone(),
            repos.start_point_writer.clone(),
            profile_repo,
            user_repo.clone(),
            settings.auth.enabled,
            settings.plugins.proxy.as_ref().map(|proxy| {
                ProxyPolicy::new(
                    proxy.allowed_service_suffixes.clone(),
                    proxy.allowed_ports.clone(),
                )
            }),
            resource_catalog.clone(),
        );

        let mut handlers = core_event_handlers(&repos);
        let mut module_routes = Vec::new();
        let mut background_tasks = Vec::new();
        let mut catalogs = std::collections::BTreeMap::new();
        let mut providers = std::collections::BTreeMap::new();
        let mut statistics_providers = std::collections::BTreeMap::new();
        for module in &modules {
            let ctx = server::module::ModuleContext {
                pool: pool.clone(),
                event_bus: event_bus.clone(),
                authorization: services.authorization.clone(),
                organizations: repos.organization_reader.clone(),
                settings: settings_repo.clone(),
                access: server::service::module_access::ModuleAccess::new(
                    module.key(),
                    services.authorization.clone(),
                    module_activation_repo.clone(),
                ),
                users: user_service.clone(),
                config: config.clone(),
                sensors: services.sensor.clone(),
                plans: services.operation_plan.clone(),
            };
            let runtime = module
                .build(ctx)
                .map_err(|err| std::io::Error::other(err.to_string()))?;
            handlers.extend(runtime.event_handlers);
            module_routes.push(runtime.routes);
            background_tasks.extend(runtime.background_tasks);
            if let Some(catalog) = runtime.targets {
                catalogs.insert(module.key(), catalog);
            }
            if let Some(provider) = runtime.planning {
                providers.insert(module.key(), provider);
            }
            if let Some(provider) = runtime.statistics {
                statistics_providers.insert(module.key(), provider);
            }
        }
        targets
            .install(catalogs)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        planning
            .install(providers)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        statistics
            .install(statistics_providers)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        event_bus
            .install(Arc::new(InMemoryEventBus::new(handlers)))
            .map_err(std::io::Error::other)?;
        for task in background_tasks {
            tokio::spawn(task);
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let (mqtt_state, mqtt_handle) =
            spawn_mqtt_subscriber(&settings, services.sensor.clone(), shutdown_rx);
        let (health_reader, readiness_reader) =
            spawn_health_probes(&pool, &settings, probe_http_client.clone(), mqtt_state).await;
        let _expiry_handle = infra::watering_status_expiry::spawn(
            services.cluster.clone(),
            repos.organization_reader.clone(),
            irrigation_settings.clone(),
            Duration::from_secs(irrigation_config.just_watered_sweep_interval_secs),
        );
        let _update_handle = infra::update_checker::spawn(
            update_checker,
            probe_http_client,
            Duration::from_secs(settings.info.update_check_interval_secs),
        );

        let plugin_proxy_service = settings.plugins.proxy.as_ref().map(|proxy| {
            if proxy.session_ttl_minutes != proxy.effective_session_ttl_minutes() {
                tracing::warn!(
                    configured = proxy.session_ttl_minutes,
                    effective = proxy.effective_session_ttl_minutes(),
                    "plugins.proxy.session_ttl_minutes lies outside 1..={} and is clamped",
                    server::configuration::MAX_PLUGIN_SESSION_TTL_MINUTES
                );
            }
            Arc::new(PluginProxyService::new(
                repos.plugin_reader.clone(),
                services.plugin_view_tickets.clone(),
                services.authorization.clone(),
                Arc::new(infra::pg_plugin_proxy_session::PgProxySessionStore::new(
                    pool.clone(),
                )),
                Arc::new(infra::plugin_proxy_session::RandomProxySessionFactory),
                ProxyPolicy::new(
                    proxy.allowed_service_suffixes.clone(),
                    proxy.allowed_ports.clone(),
                ),
                chrono::Duration::minutes(i64::from(proxy.effective_session_ttl_minutes())),
            ))
        });

        let app_origins = AppOrigins::from_settings(&settings.cors, &settings.application.base_url);
        let plugin_proxy = match &settings.plugins.proxy {
            Some(proxy) => {
                let hosts = server::http::plugin_proxy::PluginHosts::for_app(
                    &proxy.public_url,
                    &settings.application.base_url,
                )
                .map_err(|reason| std::io::Error::new(std::io::ErrorKind::InvalidInput, reason))?;
                Some(Arc::new(server::http::plugin_proxy::PluginProxy::new(
                    hosts,
                    app_origins.frame_ancestors(),
                    proxy.effective_session_ttl_minutes(),
                )))
            }
            None => None,
        };

        let state = Arc::new(AppState {
            region_service: services.region,
            tree_service: services.tree,
            sensor_service: services.sensor,
            vehicle_service: services.vehicle,
            cluster_service: services.cluster,
            comment_service: services.comment,
            operation_plan_service: services.operation_plan,
            operation_execution_service: services.operation_execution,
            evaluation_service: services.evaluation,
            user_service,
            info_provider,
            health_reader,
            readiness_reader,
            statistics_reader: repos.statistics,
            statistics,
            token_validator,
            feature_flags: FeatureFlags {
                routing_enabled: settings.routing.enabled,
                plugins_enabled: settings.plugins.enabled,
            },
            nearest_tree_limits: NearestTreeLimits {
                max_radius_meters: irrigation_config.nearest_tree_max_radius,
                default_limit: irrigation_config.nearest_tree_default_limit,
                max_limit: irrigation_config.nearest_tree_max_limit,
            },
            frontend_config_js: server::http::render_frontend_config_js(
                &settings.auth,
                &settings.analytics,
            )
            .into(),
            start_point_service: services.start_point,
            organization_service: services.organization,
            role_service: services.role,
            resource_catalog: resource_catalog.clone(),
            authorization_service: services.authorization,
            plugin_reader: repos.plugin_reader,
            plugin_writer: repos.plugin_writer,
            plugin_service: services.plugin,
            plugin_ingest_service: services.plugin_ingest,
            plugin_sensor_ingest_service: services.plugin_sensor_ingest,
            plugin_view_ticket_service: services.plugin_view_tickets,
            plugin_proxy_service,
            plugin_proxy,
            settings_reader: settings_repo.clone(),
            settings_service,
            module_activation_service,
            module_activation: module_activation_repo,
            targets,
            planning,
            app_origins,
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
            oidc_swagger: OidcSwaggerSettings {
                enabled: settings.auth.enabled,
                issuer_url: settings.auth.issuer_url,
                client_id: settings.auth.frontend_client_id,
            },
            request_timeout: Duration::from_secs(settings.application.request_timeout_secs),
            _jwks: jwks,
            shutdown_tx,
            mqtt_handle,
            module_routes,
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
            &self.oidc_swagger,
            self.request_timeout,
            self.module_routes,
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
    tree_reader: Arc<dyn irrigation_domain::tree::TreeReader>,
    tree_writer: Arc<dyn irrigation_domain::tree::TreeWriter>,
    sensor_reader: Arc<dyn domain::sensor::SensorReader>,
    sensor_writer: Arc<dyn domain::sensor::SensorWriter>,
    sensor_reading_reader: Arc<dyn domain::sensor::SensorReadingReader>,
    sensor_reading_writer: Arc<dyn domain::sensor::SensorReadingWriter>,
    sensor_model_reader: Arc<dyn domain::sensor_model::SensorModelReader>,
    vehicle_reader: Arc<dyn domain::vehicle::VehicleReader>,
    vehicle_writer: Arc<dyn domain::vehicle::VehicleWriter>,
    cluster_reader: Arc<dyn irrigation_domain::cluster::TreeClusterReader>,
    cluster_writer: Arc<dyn irrigation_domain::cluster::TreeClusterWriter>,
    comment_reader: Arc<dyn domain::comment::CommentReader>,
    comment_writer: Arc<dyn domain::comment::CommentWriter>,
    operation_plan_reader: Arc<dyn domain::operation_plan::OperationPlanReader>,
    operation_plan_writer: Arc<dyn domain::operation_plan::OperationPlanWriter>,
    evaluation: Arc<dyn irrigation_domain::evaluation::EvaluationRepository>,
    statistics: Arc<dyn StatisticsReader>,
    start_point_reader: Arc<dyn domain::start_point::StartPointReader>,
    start_point_writer: Arc<dyn domain::start_point::StartPointWriter>,
    plugin_reader: Arc<dyn domain::plugin::PluginReader>,
    plugin_writer: Arc<dyn domain::plugin::PluginWriter>,
    plugin_tree_refs: Arc<dyn irrigation_domain::plugin_refs::PluginTreeRefs>,
    plugin_view_tickets: Arc<dyn server::service::plugin_view_ticket_service::ViewTicketStore>,
}

impl Repositories {
    fn build(
        pool: &PgPool,
        sensor_offline_after: chrono::Duration,
        defect_streak: usize,
        catalog: &Arc<ResourceCatalog>,
    ) -> Self {
        let organization_repo = Arc::new(PgOrganizationRepository::new(pool.clone()));
        let role_repo = Arc::new(PgRoleRepository::new(pool.clone(), catalog.clone()));
        let region_repo = Arc::new(PgRegionRepository::new(pool.clone()));
        let tree_repo = Arc::new(PgTreeRepository::new(pool.clone()));
        let sensor_repo = Arc::new(PgSensorRepository::new(
            pool.clone(),
            sensor_offline_after,
            defect_streak,
        ));
        let vehicle_repo = Arc::new(PgVehicleRepository::new(pool.clone()));
        let cluster_repo = Arc::new(PgTreeClusterRepository::new(pool.clone()));
        let comment_repo = Arc::new(PgCommentRepository::new(pool.clone()));
        let operation_plan_repo = Arc::new(PgOperationPlanRepository::new(pool.clone()));
        let start_point_repo = Arc::new(PgStartPointRepository::new(pool.clone()));
        let plugin_repo = Arc::new(PgPluginRepository::new(pool.clone(), catalog.clone()));

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
            comment_reader: comment_repo.clone(),
            comment_writer: comment_repo,
            operation_plan_reader: operation_plan_repo.clone(),
            operation_plan_writer: operation_plan_repo,
            evaluation: Arc::new(PgEvaluationRepository::new(pool.clone())),
            statistics: Arc::new(PgStatisticsRepo::new(pool.clone())),
            start_point_reader: start_point_repo.clone(),
            start_point_writer: start_point_repo,
            plugin_reader: plugin_repo.clone(),
            plugin_writer: plugin_repo.clone(),
            plugin_tree_refs: plugin_repo,
            plugin_view_tickets: Arc::new(
                server::infra::pg_plugin_view_ticket::PgViewTicketStore::new(pool.clone()),
            ),
        }
    }
}

struct Services {
    region: Arc<RegionService>,
    tree: Arc<TreeService>,
    sensor: Arc<SensorService>,
    vehicle: Arc<VehicleService>,
    cluster: Arc<ClusterService>,
    comment: Arc<CommentService>,
    operation_plan: Arc<OperationPlanService>,
    operation_execution: Arc<OperationExecutionService>,
    evaluation: Arc<EvaluationService>,
    start_point: Arc<StartPointService>,
    organization: Arc<OrganizationService>,
    role: Arc<RoleService>,
    authorization: Arc<AuthorizationService>,
    plugin: Arc<PluginService>,
    plugin_ingest: Arc<PluginIngestService>,
    plugin_sensor_ingest: Arc<PluginSensorIngestService>,
    plugin_view_tickets: Arc<PluginViewTicketService>,
}

impl Services {
    #[allow(clippy::too_many_arguments)]
    fn build(
        repos: &Repositories,
        event_bus: Arc<dyn EventBus>,
        targets: Arc<server::service::targets::TargetRegistry>,
        planning: Arc<PlanningRegistry>,
        module_activation: Arc<dyn domain::module::ModuleActivationReader>,
        route_optimizer: Option<Arc<dyn domain::routing::RouteOptimizer>>,
        start_point_reader: Arc<dyn domain::start_point::StartPointReader>,
        start_point_writer: Arc<dyn domain::start_point::StartPointWriter>,
        profile_reader: Arc<dyn domain::user::UserProfileReader>,
        user_repo: Arc<dyn domain::user::UserRepository>,
        auth_enabled: bool,
        proxy_policy: Option<ProxyPolicy>,
        resource_catalog: Arc<ResourceCatalog>,
    ) -> Self {
        let authorization = Arc::new(AuthorizationService::new(
            repos.organization_reader.clone(),
            repos.role_reader.clone(),
            auth_enabled,
        ));
        let plugin = Arc::new(PluginService::new(
            repos.plugin_reader.clone(),
            repos.plugin_writer.clone(),
            authorization.clone(),
            Arc::new(server::infra::plugin_key::RandomPluginKeyFactory),
            proxy_policy,
            resource_catalog.clone(),
        ));
        let tree = Arc::new(TreeService::new(
            repos.tree_reader.clone(),
            repos.tree_writer.clone(),
            repos.cluster_reader.clone(),
            repos.sensor_reader.clone(),
            repos.sensor_writer.clone(),
            event_bus.clone(),
        ));
        // Reuses `TreeService::delete` for its own delete flow so `TreeDeleted`
        // fires and cluster centroid/status stay in sync, instead of a second
        // copy of that logic.
        let plugin_ingest = Arc::new(PluginIngestService::new(
            repos.tree_reader.clone(),
            repos.tree_writer.clone(),
            repos.plugin_tree_refs.clone(),
            tree.clone(),
            event_bus.clone(),
            authorization.clone(),
        ));
        let sensor = Arc::new(SensorService::new(
            repos.sensor_reader.clone(),
            repos.sensor_writer.clone(),
            repos.sensor_reading_reader.clone(),
            repos.sensor_reading_writer.clone(),
            repos.sensor_model_reader.clone(),
            repos.tree_reader.clone(),
            repos.cluster_reader.clone(),
            targets.clone(),
            module_activation.clone(),
            event_bus.clone(),
        ));
        let operation_plan = Arc::new(OperationPlanService::new(
            repos.operation_plan_reader.clone(),
            repos.operation_plan_writer.clone(),
            repos.vehicle_reader.clone(),
            event_bus.clone(),
            route_optimizer,
            start_point_reader.clone(),
            repos.comment_writer.clone(),
            planning,
            module_activation.clone(),
        ));
        let plugin_sensor_ingest = Arc::new(PluginSensorIngestService::new(
            sensor.clone(),
            authorization.clone(),
        ));
        Self {
            region: Arc::new(RegionService::new(
                repos.region_reader.clone(),
                repos.region_writer.clone(),
            )),
            tree,
            sensor,
            vehicle: Arc::new(VehicleService::new(
                repos.vehicle_reader.clone(),
                repos.vehicle_writer.clone(),
            )),
            cluster: Arc::new(ClusterService::new(
                repos.cluster_reader.clone(),
                repos.cluster_writer.clone(),
                repos.tree_reader.clone(),
                repos.tree_writer.clone(),
                repos.sensor_reader.clone(),
                repos.sensor_writer.clone(),
                event_bus.clone(),
                repos.comment_writer.clone(),
                operation_plan.clone(),
            )),
            comment: Arc::new(CommentService::new(
                repos.comment_reader.clone(),
                repos.comment_writer.clone(),
                targets.clone(),
                module_activation.clone(),
            )),
            operation_execution: Arc::new(OperationExecutionService::new(
                repos.operation_plan_reader.clone(),
                repos.operation_plan_writer.clone(),
                event_bus.clone(),
            )),
            operation_plan,
            evaluation: Arc::new(EvaluationService::new(repos.evaluation.clone())),
            start_point: Arc::new(StartPointService::new(
                start_point_reader.clone(),
                start_point_writer,
            )),
            organization: Arc::new(OrganizationService::new(
                repos.organization_reader.clone(),
                repos.organization_writer.clone(),
                repos.role_reader.clone(),
                profile_reader.clone(),
                user_repo,
                event_bus.clone(),
            )),
            role: Arc::new(RoleService::new(
                repos.role_reader.clone(),
                repos.role_writer.clone(),
                profile_reader,
                event_bus,
            )),
            authorization,
            plugin,
            plugin_ingest,
            plugin_sensor_ingest,
            plugin_view_tickets: Arc::new(PluginViewTicketService::new(
                repos.plugin_view_tickets.clone(),
                Arc::new(server::infra::plugin_view_ticket::RandomViewTicketFactory),
            )),
        }
    }
}

/// Turns the instance's configured values into the floor every resolution
/// starts from. A value outside the domain's range is a broken deployment,
/// not a request, so the boot fails with the offending field named rather
/// than panicking on the way up or surfacing on the first read.
fn instance_defaults(
    settings: &Settings,
) -> Result<domain::settings::InstanceDefaults, std::io::Error> {
    use domain::settings::{
        DefectStreak, InstanceDefaults, MapBounds, MapView, SensorOfflineAfter, ZoomLevel,
    };
    use domain::shared::{coordinates::Coordinate, geo::BoundingBox};

    let [lat, lng] = settings.map.center;
    let [sw_lat, sw_lng, ne_lat, ne_lng] = settings.map.bbox;

    let rejected = |err: domain::shared::error::ValidationError| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid configuration: {err}"),
        )
    };

    Ok(InstanceDefaults {
        sensor_offline_after: SensorOfflineAfter::new(
            i64::try_from(settings.sensor.offline_after_secs).unwrap_or(i64::MAX),
        )
        .map_err(rejected)?,
        defect_streak: DefectStreak::new(
            i32::try_from(settings.sensor.defect_streak).unwrap_or(i32::MAX),
        )
        .map_err(rejected)?,
        map_view: MapView::new(
            Coordinate::new(lat, lng).map_err(rejected)?,
            Some(
                MapBounds::new(
                    BoundingBox::try_new(sw_lat, sw_lng, ne_lat, ne_lng).map_err(rejected)?,
                    ZoomLevel::new(settings.map.min_zoom).map_err(rejected)?,
                    ZoomLevel::new(settings.map.max_zoom).map_err(rejected)?,
                )
                .map_err(rejected)?,
            ),
        )
        .map_err(rejected)?,
    })
}

fn build_http_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .expect("reqwest client must build")
}

fn core_event_handlers(repos: &Repositories) -> Vec<Arc<dyn EventHandler>> {
    vec![
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
        Arc::new(ClusterJustWateredHandler::new(
            repos.cluster_reader.clone(),
            repos.cluster_writer.clone(),
            repos.tree_reader.clone(),
            repos.tree_writer.clone(),
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
    ]
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
