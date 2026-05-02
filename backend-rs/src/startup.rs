use std::sync::Arc;

use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;

use crate::{
    configuration::{AuthSettings, CorsSettings, DatabaseSettings, Settings},
    domain::auth::AuthRepository,
    domain::user::UserRepository,
    http::{AppState, auth::AuthLayer, router},
    infra::{
        keycloak::{JwksProvider, KeycloakAuthRepository, KeycloakClient, KeycloakUserRepository},
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
        auth_service::{AuthService, AuthServiceConfig},
        cluster_service::ClusterService,
        evaluation_service::EvaluationService,
        event_bus::{EventBus, InMemoryEventBus},
        handlers::cluster_recalc::ClusterRecalculationHandler,
        region_service::RegionService,
        sensor_service::SensorService,
        tree_service::TreeService,
        user_service::UserService,
        vehicle_service::VehicleService,
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
        Self::build_with_pool(
            pool,
            &address,
            config.application.base_url,
            config.cors,
            config.auth,
        )
        .await
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
        let tree_repo: Arc<dyn crate::domain::tree::TreeRepository> =
            Arc::new(PgTreeRepository::new(pool.clone()));
        let sensor_repo: Arc<dyn crate::domain::sensor::SensorRepository> =
            Arc::new(PgSensorRepository::new(pool.clone()));
        let vehicle_repo: Arc<dyn crate::domain::vehicle::VehicleRepository> =
            Arc::new(PgVehicleRepository::new(pool.clone()));
        let cluster_repo: Arc<dyn crate::domain::cluster::TreeClusterRepository> =
            Arc::new(PgTreeClusterRepository::new(pool.clone()));
        let watering_plan_repo: Arc<dyn crate::domain::watering_plan::WateringPlanRepository> =
            Arc::new(PgWateringPlanRepository::new(pool.clone()));
        let evaluation_repo: Arc<dyn crate::domain::evaluation::EvaluationRepository> =
            Arc::new(PgEvaluationRepository::new(pool));

        let kc_client = Arc::new(
            KeycloakClient::new(&auth)
                .map_err(|e| std::io::Error::other(format!("keycloak client init: {e}")))?,
        );
        let jwks = Arc::new(JwksProvider::new(&kc_client, &auth));
        if auth.enabled {
            // Soft-fail: dev environments without a running Keycloak can still boot;
            // the background refresher will pick up keys once it comes online.
            if let Err(err) = jwks.refresh_now().await {
                tracing::warn!(error = %err, "initial JWKS refresh failed; will retry in background");
            }
            jwks.spawn_background_refresh();
        }

        let auth_repo: Arc<dyn AuthRepository> =
            Arc::new(KeycloakAuthRepository::new(kc_client.clone()));
        let user_repo: Arc<dyn UserRepository> =
            Arc::new(KeycloakUserRepository::new(kc_client.clone()));

        let auth_service_config = AuthServiceConfig::from_settings(&auth)
            .map_err(|e| std::io::Error::other(format!("auth service config: {e}")))?;
        let auth_service = Arc::new(AuthService::new(auth_repo, auth_service_config));
        let user_service = Arc::new(UserService::new(user_repo, auth.enabled));

        // Event handlers
        let cluster_recalc_handler = Arc::new(ClusterRecalculationHandler::new(
            cluster_repo.clone(),
            region_reader.clone(),
        ));

        // Event bus
        let event_bus: Arc<dyn EventBus> =
            Arc::new(InMemoryEventBus::new(vec![cluster_recalc_handler]));

        // Domain services
        let region_service = Arc::new(RegionService::new(region_reader, region_writer));
        let tree_service = Arc::new(TreeService::new(tree_repo.clone(), event_bus.clone()));
        let sensor_service = Arc::new(SensorService::new(
            sensor_repo,
            tree_repo.clone(),
            event_bus.clone(),
        ));
        let vehicle_service = Arc::new(VehicleService::new(vehicle_repo));
        let cluster_service = Arc::new(ClusterService::new(
            cluster_repo,
            tree_repo,
            event_bus.clone(),
        ));
        let watering_plan_service =
            Arc::new(WateringPlanService::new(watering_plan_repo, event_bus));
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
            evaluation_service,
            auth_service,
            user_service,
            info_provider,
        });

        let auth_layer = AuthLayer::new(jwks.clone(), &auth);

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
