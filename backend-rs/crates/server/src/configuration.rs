use std::path::PathBuf;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use url::Url;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub log: LogSettings,
    pub cors: CorsSettings,
    pub auth: AuthSettings,
    #[serde(default)]
    pub mqtt: MqttSettings,
    #[serde(default)]
    pub map: MapSettings,
    #[serde(default)]
    pub info: InfoSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub max_connections: u32,
    pub log_statements_level: LogLevel,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub slow_query_threshold_ms: u64,
}

impl DatabaseSettings {
    pub fn connection_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .database(&self.database_name)
            .ssl_mode(ssl_mode)
            .log_statements(self.log_statements_level.into())
            .log_slow_statements(
                log::LevelFilter::Warn,
                Duration::from_millis(self.slow_query_threshold_ms),
            )
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    #[serde(deserialize_with = "deserialize_url")]
    pub base_url: Url,
    #[serde(default)]
    pub environment: Environment,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    Url::parse(&raw).map_err(serde::de::Error::custom)
}

#[derive(serde::Deserialize, Clone)]
pub struct LogSettings {
    pub level: String,
    pub format: LogFormat,
}

#[derive(serde::Deserialize, Clone)]
pub struct CorsSettings {
    pub allowed_origins: Vec<String>,
}

#[derive(serde::Deserialize, Clone, Default)]
pub struct MqttSettings {
    /// When false the subscriber task is not started. Defaults to `false` so
    /// integration tests and dev runs without a broker stay green.
    #[serde(default)]
    pub enabled: bool,
    /// Broker URL, e.g. `mqtt://localhost:1883` or `mqtts://broker:8883`.
    #[serde(default)]
    pub broker_url: String,
    /// Stable client identifier — defaults to `green-ecolution`.
    #[serde(default = "default_client_id")]
    pub client_id: String,
    /// Topic filter to subscribe to (single string; wildcards `+`/`#` allowed).
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<SecretString>,
    /// Keep-alive in seconds for the broker connection.
    #[serde(default = "default_keep_alive_secs")]
    pub keep_alive_secs: u16,
}

fn default_client_id() -> String {
    "green-ecolution".to_string()
}

fn default_keep_alive_secs() -> u16 {
    30
}

#[derive(serde::Deserialize, Clone)]
pub struct MapSettings {
    pub center: [f64; 2],
    pub bbox: [f64; 4],
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            center: [54.792277136221905, 9.43580607453268],
            bbox: [54.714822, 9.285796, 54.860127, 9.583800],
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct InfoSettings {
    #[serde(default = "default_health_check_interval_secs")]
    pub health_check_interval_secs: u64,
    #[serde(default = "default_health_probe_timeout_secs")]
    pub health_probe_timeout_secs: u64,
    #[serde(default)]
    pub update_check_repo: Option<String>,
    #[serde(default = "default_update_check_interval_secs")]
    pub update_check_interval_secs: u64,
    #[serde(
        default = "default_repository_url",
        deserialize_with = "deserialize_url"
    )]
    pub repository_url: Url,
}

impl Default for InfoSettings {
    fn default() -> Self {
        Self {
            health_check_interval_secs: default_health_check_interval_secs(),
            health_probe_timeout_secs: default_health_probe_timeout_secs(),
            update_check_repo: None,
            update_check_interval_secs: default_update_check_interval_secs(),
            repository_url: default_repository_url(),
        }
    }
}

fn default_health_check_interval_secs() -> u64 {
    30
}
fn default_health_probe_timeout_secs() -> u64 {
    5
}
fn default_update_check_interval_secs() -> u64 {
    86_400
}
fn default_repository_url() -> Url {
    Url::parse("https://github.com/green-ecolution/backend-rs/")
        .expect("default repository_url must parse")
}

#[derive(serde::Deserialize, Clone)]
pub struct AuthSettings {
    pub enabled: bool,
    pub issuer_url: String,
    pub frontend_client_id: String,
    pub backend_client_id: String,
    pub backend_client_secret: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub jwks_refresh_interval_secs: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub jwks_refresh_timeout_secs: u64,
    pub default_redirect_url: String,
    #[serde(default)]
    pub expected_audience: Option<String>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Json,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Errors that can occur while loading the application configuration.
///
/// Construction is intentionally separated from `config::ConfigError` so the
/// binary can render actionable startup diagnostics (working directory,
/// `APP_ENVIRONMENT`, expected file layout) before tracing is initialized.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not determine the current working directory")]
    CurrentDir(#[source] std::io::Error),

    #[error("APP_ENVIRONMENT={value:?} is not a supported environment: {reason}")]
    InvalidEnvironment { value: String, reason: String },

    #[error("config directory does not exist: {0}")]
    ConfigDirMissing(PathBuf),

    #[error(transparent)]
    Source(#[from] config::ConfigError),
}

impl Settings {
    /// Build a `Settings` value for integration tests. Database settings are
    /// dummy — the test pool is created separately and passed to
    /// `Application::build_with_pool`.
    pub fn for_test(auth: AuthSettings) -> Self {
        Self {
            database: DatabaseSettings {
                username: "postgres".into(),
                password: SecretString::from("postgres".to_string()),
                port: 5432,
                host: "127.0.0.1".into(),
                database_name: "postgres".into(),
                require_ssl: false,
                max_connections: 1,
                log_statements_level: LogLevel::Warn,
                slow_query_threshold_ms: 1000,
            },
            application: ApplicationSettings {
                port: 0,
                host: "127.0.0.1".into(),
                base_url: Url::parse("http://127.0.0.1").expect("test base_url"),
                environment: Environment::Local,
            },
            log: LogSettings {
                level: "warn".into(),
                format: LogFormat::Pretty,
            },
            cors: CorsSettings {
                allowed_origins: vec!["*".into()],
            },
            auth,
            mqtt: MqttSettings::default(),
            map: MapSettings::default(),
            info: InfoSettings::default(),
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().map_err(ConfigError::CurrentDir)?;
    let configuration_dir = base_path.join("config");

    if !configuration_dir.is_dir() {
        return Err(ConfigError::ConfigDirMissing(configuration_dir));
    }

    let env_value = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());
    let environment: Environment =
        env_value
            .clone()
            .try_into()
            .map_err(|reason| ConfigError::InvalidEnvironment {
                value: env_value,
                reason,
            })?;

    let env_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_dir.join("base.yaml")))
        .add_source(config::File::from(configuration_dir.join(env_filename)))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let mut settings: Settings = settings.try_deserialize()?;
    // `APP_ENVIRONMENT` selects the yaml file — keep `application.environment`
    // in sync so consumers (e.g. version suffix logic) don't have to read the
    // env var directly.
    settings.application.environment = environment;
    Ok(settings)
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Local,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use `local`, `staging`, or `production`."
            )),
        }
    }
}
