use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use url::Url;

use domain::RepositoryError;
use domain::info::{App, Git, Map, Server, SystemInfoProvider, VersionInfo};

use crate::configuration::{Environment, Settings};
use crate::infra::update_checker::UpdateChecker;

pub struct DefaultSystemInfoProvider {
    start_time: Instant,
    version: String,
    rust_version: String,
    build_time: DateTime<Utc>,
    git: Git,
    map: Map,
    release_url: Url,
    server_base_url: Url,
    bind_interface: String,
    bind_port: u16,
    is_stage: bool,
    update_checker: Arc<UpdateChecker>,
}

impl DefaultSystemInfoProvider {
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        use crate::configuration::{ApplicationSettings, InfoSettings, MapSettings};
        let stub_settings = Settings {
            database: crate::configuration::DatabaseSettings {
                username: "postgres".into(),
                password: secrecy::SecretString::from("postgres".to_string()),
                port: 5432,
                host: "127.0.0.1".into(),
                database_name: "postgres".into(),
                require_ssl: false,
                max_connections: 1,
                log_statements_level: crate::configuration::LogLevel::Warn,
                slow_query_threshold_ms: 1000,
            },
            application: ApplicationSettings {
                port: 0,
                host: "127.0.0.1".into(),
                base_url: "http://127.0.0.1".into(),
                environment: Environment::Local,
            },
            log: crate::configuration::LogSettings {
                level: "warn".into(),
                format: crate::configuration::LogFormat::Pretty,
            },
            cors: crate::configuration::CorsSettings {
                allowed_origins: vec!["*".into()],
            },
            auth: crate::configuration::AuthSettings {
                enabled: false,
                issuer_url: "http://127.0.0.1".into(),
                frontend_client_id: "test".into(),
                backend_client_id: "test".into(),
                backend_client_secret: secrecy::SecretString::from("secret".to_string()),
                jwks_refresh_interval_secs: 60,
                jwks_refresh_timeout_secs: 5,
                default_redirect_url: "http://127.0.0.1/cb".into(),
                expected_audience: None,
            },
            mqtt: crate::configuration::MqttSettings::default(),
            map: MapSettings::default(),
            info: InfoSettings::default(),
        };
        let checker = Arc::new(UpdateChecker::new(
            env!("CARGO_PKG_VERSION").to_string(),
            None,
        ));
        Self::new(&stub_settings, checker)
    }

    pub fn new(settings: &Settings, update_checker: Arc<UpdateChecker>) -> Self {
        let repository_url: Url = settings
            .info
            .repository_url
            .parse()
            .expect("info.repository_url must be a valid URL");
        let release_url = repository_url
            .join("releases/")
            .expect("releases/ must be appendable to repository_url");

        Self {
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: env!("GE_RUSTC_VERSION").to_string(),
            build_time: DateTime::parse_from_rfc3339(env!("GE_BUILD_TIME"))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            git: Git {
                branch: option_env!("GE_GIT_BRANCH").unwrap_or("unknown").to_string(),
                commit: option_env!("GE_GIT_COMMIT").unwrap_or("unknown").to_string(),
                repository: repository_url,
            },
            map: Map {
                center: settings.map.center,
                bbox: settings.map.bbox,
            },
            release_url,
            server_base_url: settings
                .application
                .base_url
                .parse()
                .expect("application.base_url must be a valid URL"),
            bind_interface: settings.application.host.clone(),
            bind_port: settings.application.port,
            is_stage: matches!(settings.application.environment, Environment::Staging),
            update_checker,
        }
    }
}

#[async_trait::async_trait]
impl SystemInfoProvider for DefaultSystemInfoProvider {
    async fn app_info(&self) -> Result<App, RepositoryError> {
        let latest = self.update_checker.latest().await;
        let update_available = self.update_checker.update_available().await;

        Ok(App {
            version: self.version.clone(),
            version_info: VersionInfo {
                current: self.version.clone(),
                latest,
                update_available,
                is_development: cfg!(debug_assertions),
                is_stage: self.is_stage,
                release_url: self.release_url.clone(),
            },
            rust_version: self.rust_version.clone(),
            build_time: self.build_time,
            git: self.git.clone(),
            server: self.server_info().await?,
            map: self.map,
            services: vec![],
        })
    }

    async fn map_info(&self) -> Result<Map, RepositoryError> {
        Ok(self.map)
    }

    async fn server_info(&self) -> Result<Server, RepositoryError> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(Server {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname,
            url: self.server_base_url.clone(),
            port: self.bind_port,
            interface: self.bind_interface.clone(),
            uptime: self.start_time.elapsed(),
        })
    }
}
