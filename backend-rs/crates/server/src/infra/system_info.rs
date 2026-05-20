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
    rust_channel: String,
    rust_edition: String,
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
    pub fn new(settings: &Settings, update_checker: Arc<UpdateChecker>) -> Self {
        let repository_url: Url = settings
            .info
            .repository_url
            .parse()
            .expect("info.repository_url must be a valid URL");
        let release_url = repository_url
            .join("releases/")
            .expect("releases/ must be appendable to repository_url");

        let commit = option_env!("GE_GIT_COMMIT").unwrap_or("unknown");
        let is_stage = matches!(settings.application.environment, Environment::Staging);
        let version = build_display_version(env!("CARGO_PKG_VERSION"), commit, is_stage);

        Self {
            start_time: Instant::now(),
            version,
            rust_version: env!("GE_RUSTC_VERSION").to_string(),
            rust_channel: env!("GE_RUST_CHANNEL").to_string(),
            rust_edition: env!("GE_RUST_EDITION").to_string(),
            build_time: DateTime::parse_from_rfc3339(env!("GE_BUILD_TIME"))
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            git: Git {
                branch: option_env!("GE_GIT_BRANCH")
                    .unwrap_or("unknown")
                    .to_string(),
                commit: commit.to_string(),
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
            is_stage,
            update_checker,
        }
    }
}

fn build_display_version(raw: &str, commit: &str, is_stage: bool) -> String {
    if cfg!(debug_assertions) {
        format!("{raw}+dev.{commit}")
    } else if is_stage {
        format!("{raw}+stage.{commit}")
    } else {
        raw.to_string()
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
            rust_channel: self.rust_channel.clone(),
            rust_edition: self.rust_edition.clone(),
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
