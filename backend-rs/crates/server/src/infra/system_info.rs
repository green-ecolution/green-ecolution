use std::time::Instant;

use chrono::Utc;

use domain::RepositoryError;
use domain::info::{App, Git, Map, Server, SystemInfoProvider, VersionInfo};

pub struct DefaultSystemInfoProvider {
    start_time: Instant,
    version: String,
    map: Map,
}

impl DefaultSystemInfoProvider {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            map: Map {
                center: [54.792277136221905, 9.43580607453268],
                bbox: [54.714822, 9.285796, 54.860127, 9.583800],
            },
        }
    }
}

impl Default for DefaultSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SystemInfoProvider for DefaultSystemInfoProvider {
    async fn app_info(&self) -> Result<App, RepositoryError> {
        Ok(App {
            version: self.version.clone(),
            version_info: VersionInfo {
                current: self.version.clone(),
                latest: self.version.clone(),
                update_available: false,
                is_development: cfg!(debug_assertions),
                is_stage: false,
                release_url: "https://github.com/green-ecolution/backend-rs/releases"
                    .parse()
                    .unwrap(),
            },
            rust_version: format!("rustc {}", env!("CARGO_PKG_RUST_VERSION")),
            build_time: Utc::now(),
            git: Git {
                branch: option_env!("GE_GIT_BRANCH")
                    .unwrap_or("unknown")
                    .to_string(),
                commit: option_env!("GE_GIT_COMMIT")
                    .unwrap_or("unknown")
                    .to_string(),
                repository: "https://github.com/green-ecolution/backend-rs"
                    .parse()
                    .unwrap(),
            },
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
            url: "http://localhost:3000".parse().unwrap(),
            port: 3000,
            interface: "0.0.0.0".to_string(),
            uptime: self.start_time.elapsed(),
        })
    }
}
