use std::sync::{Arc, OnceLock};

use domain::{
    events::SensorReadings,
    sensor::{SensorId, data::Watermark, repository::NormalizedValue},
    sensor_model::SensorAbilityName,
};
use rust_decimal::Decimal;
use secrecy::SecretString;
use server::{
    configuration::{AuthSettings, Settings},
    http::AppState,
    service::{ServiceError, sensor_service::ReadingIngest},
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

    /// Mirrors what `infra::mqtt::build_eco_drizzler` produces after parsing:
    /// three watermarks (30/60/90 cm) plus temperature and humidity at 15 cm.
    pub async fn ingest_ecodrizzler(
        &self,
        sensor_id: &str,
        centibar: i32,
    ) -> Result<(), ServiceError> {
        let model_id = self.ecodrizzler_model_id().await;
        let model = self
            .state
            .sensor_service
            .model_by_id(domain::Id::new(model_id))
            .await
            .expect("ecodrizzler model exists");

        let watermarks = vec![
            Watermark {
                depth: 30,
                resistance: 0,
                centibar,
            },
            Watermark {
                depth: 60,
                resistance: 0,
                centibar,
            },
            Watermark {
                depth: 90,
                resistance: 0,
                centibar,
            },
        ];
        let mut normalized: Vec<NormalizedValue> = watermarks
            .iter()
            .filter_map(|w| {
                model
                    .ability_id_for(SensorAbilityName::SoilTension, w.depth)
                    .map(|id| NormalizedValue {
                        model_ability_id: id,
                        value: Decimal::from(w.centibar),
                    })
            })
            .collect();
        if let Some(id) = model.ability_id_for(SensorAbilityName::Temperature, 15) {
            normalized.push(NormalizedValue {
                model_ability_id: id,
                value: Decimal::from(18),
            });
        }
        if let Some(id) = model.ability_id_for(SensorAbilityName::Humidity, 15) {
            normalized.push(NormalizedValue {
                model_ability_id: id,
                value: Decimal::new(4, 1),
            });
        }

        let raw_payload = serde_json::json!({
            "device": sensor_id,
            "watermarks": &watermarks,
        });
        self.state
            .sensor_service
            .ingest_reading(ReadingIngest {
                sensor_id: SensorId::new(sensor_id).expect("valid sensor id"),
                raw_payload,
                normalized,
                typed: SensorReadings::Watermarks(watermarks),
            })
            .await
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
    let mut settings = Settings::for_test(auth);
    settings.info.health_check_interval_secs = 1;
    settings.info.update_check_repo = None;
    spawn_with_settings(settings).await
}

pub async fn spawn_app_with_routing(streamlet_url: &str) -> TestApp {
    let mut settings = Settings::for_test(disabled_auth_settings());
    settings.info.health_check_interval_secs = 1;
    settings.info.update_check_repo = None;
    settings.routing.enabled = true;
    settings.routing.streamlet_url = streamlet_url.to_string();
    let app = spawn_with_settings(settings).await;
    seed_routing_depots(&app.db_pool).await;
    app
}

/// Seeds the same start points the production seed file provides, plus a
/// "Depot Nord" the routing tests select by name (lat≈54.81).
async fn seed_routing_depots(pool: &sqlx::PgPool) {
    sqlx::query(
        r#"INSERT INTO depots (id, name, latitude, longitude, geometry, watering_point, is_default) VALUES
           (gen_random_uuid(), 'Betriebshof Schleswiger Straße', 54.76879146396569, 9.434803531218018, ST_SetSRID(ST_MakePoint(9.434803531218018, 54.76879146396569), 4326), TRUE, TRUE),
           (gen_random_uuid(), 'Klärwerk Kielseng', 54.80518123149477, 9.447145106541388, ST_SetSRID(ST_MakePoint(9.447145106541388, 54.80518123149477), 4326), TRUE, FALSE),
           (gen_random_uuid(), 'Depot Nord', 54.81, 9.45, ST_SetSRID(ST_MakePoint(9.45, 54.81), 4326), FALSE, FALSE)"#,
    )
    .execute(pool)
    .await
    .expect("failed to seed depots");
}

async fn spawn_with_settings(settings: Settings) -> TestApp {
    let container = shared_container().await;
    let (_db_name, db_pool) = create_test_database(container.host_port).await;

    let app = Application::build_with_pool(db_pool.clone(), "127.0.0.1:0", settings)
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
