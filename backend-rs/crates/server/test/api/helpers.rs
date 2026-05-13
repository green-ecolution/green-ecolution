use std::sync::{Arc, OnceLock};

use domain::sensor::data::MqttPayload;
use secrecy::SecretString;
use server::{
    configuration::{AuthSettings, CorsSettings},
    http::AppState,
    infra::system_info::DefaultSystemInfoProvider,
    service::ServiceError,
    startup::Application,
};
use sqlx::{Connection, Executor, PgConnection, PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};
use tokio::sync::OnceCell;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub state: Arc<AppState>,
}

impl TestApp {
    /// Look up the seeded `EcoDrizzler` sensor model UUID. Migrations seed two
    /// well-known models by name — tests reference them through this helper
    /// since the integer ids no longer exist after the UUID migration.
    pub async fn ecodrizzler_model_id(&self) -> Uuid {
        sqlx::query_scalar!(r#"SELECT id FROM sensor_models WHERE name = 'EcoDrizzler'"#)
            .fetch_one(&self.db_pool)
            .await
            .expect("EcoDrizzler model must exist after migrations")
    }

    /// Look up the seeded `GES-1000` sensor model UUID.
    pub async fn ges_1000_model_id(&self) -> Uuid {
        sqlx::query_scalar!(r#"SELECT id FROM sensor_models WHERE name = 'GES-1000'"#)
            .fetch_one(&self.db_pool)
            .await
            .expect("GES-1000 model must exist after migrations")
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        reqwest::Client::new()
            .get(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn post_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}{}", self.address, path))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn put_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .put(format!("{}{}", self.address, path))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn delete(&self, path: &str) -> reqwest::Response {
        reqwest::Client::new()
            .delete(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("failed to execute request")
    }

    /// Drives the MQTT ingest path directly without spinning up a broker.
    /// `payload` is JSON in the same shape the production MQTT subscriber
    /// builds before calling [`SensorService::handle_message`].
    pub async fn handle_mqtt_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<(), ServiceError> {
        let typed: MqttPayload = serde_json::from_value(payload)
            .expect("test payload must deserialise into MqttPayload");
        self.state.sensor_service.handle_message(typed).await
    }
}

struct SharedContainer {
    _container: ContainerAsync<GenericImage>,
    host_port: u16,
}

static CONTAINER: OnceCell<SharedContainer> = OnceCell::const_new();
static CONTAINER_ID: OnceLock<String> = OnceLock::new();

// `static` destructors don't run on process exit, and the testcontainers
// `watchdog` feature only fires on SIGTERM/SIGINT/SIGQUIT — neither covers a
// clean test-runner exit. Register a libc `atexit` hook that force-removes the
// container via the docker CLI as a synchronous fallback.
extern "C" fn cleanup_container_at_exit() {
    if let Some(id) = CONTAINER_ID.get() {
        let _ = std::process::Command::new("docker")
            .args(["rm", "-f", id])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

async fn shared_container() -> &'static SharedContainer {
    CONTAINER
        .get_or_init(|| async {
            let image = GenericImage::new("postgis/postgis", "17-3.5")
                .with_exposed_port(5432.into())
                .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
                    "database system is ready to accept connections",
                ))
                .with_env_var("POSTGRES_USER", "postgres")
                .with_env_var("POSTGRES_PASSWORD", "postgres")
                .with_env_var("POSTGRES_DB", "postgres");

            let container = image
                .start()
                .await
                .expect("failed to start postgis container");

            let host_port = container
                .get_host_port_ipv4(5432)
                .await
                .expect("failed to get postgres port");

            CONTAINER_ID
                .set(container.id().to_string())
                .expect("container id already set");
            unsafe { libc::atexit(cleanup_container_at_exit) };

            SharedContainer {
                _container: container,
                host_port,
            }
        })
        .await
}

async fn create_test_database(host_port: u16) -> (String, PgPool) {
    let db_name = format!("test_{}", Uuid::new_v4().simple());

    let admin_url = format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres");
    let mut admin = PgConnection::connect(&admin_url)
        .await
        .expect("failed to connect to admin database");
    admin
        .execute(format!(r#"CREATE DATABASE "{db_name}""#).as_str())
        .await
        .expect("failed to create test database");

    let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{host_port}/{db_name}");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    (db_name, pool)
}

// `issuer_url` must still match `/realms/<name>` even when `enabled = false`
// — `KeycloakClient::new` parses it at boot regardless.
pub fn disabled_auth_settings() -> AuthSettings {
    AuthSettings {
        enabled: false,
        issuer_url: "http://127.0.0.1:1/realms/test".to_string(),
        frontend_client_id: "frontend".to_string(),
        backend_client_id: "backend".to_string(),
        backend_client_secret: SecretString::from("test".to_string()),
        jwks_refresh_interval_secs: 60,
        jwks_refresh_timeout_secs: 5,
        default_redirect_url: "http://127.0.0.1/cb".to_string(),
        expected_audience: None,
    }
}

pub async fn spawn_app() -> TestApp {
    spawn_app_with_auth(disabled_auth_settings()).await
}

pub async fn spawn_app_with_auth(auth: AuthSettings) -> TestApp {
    let container = shared_container().await;
    let (_db_name, db_pool) = create_test_database(container.host_port).await;

    let info_provider: Arc<dyn domain::info::SystemInfoProvider> =
        Arc::new(DefaultSystemInfoProvider::new_for_test());
    let app = Application::build_with_pool(
        db_pool.clone(),
        "127.0.0.1:0",
        "http://127.0.0.1".to_string(),
        CorsSettings {
            allowed_origins: vec!["*".to_string()],
        },
        auth,
        info_provider,
    )
    .await
    .expect("failed to build application");
    let port = app.port();
    let state = app.state();
    tokio::spawn(app.run_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        port,
        db_pool,
        state,
    }
}
