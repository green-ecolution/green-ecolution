use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use url::Url;

use domain::RepositoryError;
use domain::info::{App, Git, Map, Server, SystemInfoProvider, VersionInfo};

use crate::configuration::{Environment, Settings};
use crate::infra::update_checker::UpdateChecker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildKind {
    Dev,
    Stage,
    Release,
}

impl BuildKind {
    fn current(is_stage: bool) -> Self {
        if cfg!(debug_assertions) {
            Self::Dev
        } else if is_stage {
            Self::Stage
        } else {
            Self::Release
        }
    }

    fn is_development(self) -> bool {
        matches!(self, Self::Dev)
    }

    fn is_stage(self) -> bool {
        matches!(self, Self::Stage)
    }
}

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
    build_kind: BuildKind,
    update_checker: Arc<UpdateChecker>,
}

impl DefaultSystemInfoProvider {
    pub fn new(settings: &Settings, update_checker: Arc<UpdateChecker>) -> Self {
        let repository_url = settings.info.repository_url.clone();
        let release_url = repository_url
            .join("releases/")
            .expect("releases/ must be appendable to repository_url");

        let commit = option_env!("GE_GIT_COMMIT").unwrap_or("unknown");
        let is_stage = matches!(settings.application.environment, Environment::Staging);
        let build_kind = BuildKind::current(is_stage);
        let version = build_display_version(env!("CARGO_PKG_VERSION"), commit, build_kind);

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
            server_base_url: settings.application.base_url.clone(),
            bind_interface: settings.application.host.clone(),
            bind_port: settings.application.port,
            build_kind,
            update_checker,
        }
    }
}

fn build_display_version(raw: &str, commit: &str, kind: BuildKind) -> String {
    match kind {
        BuildKind::Dev => format!("{raw}+dev.{commit}"),
        BuildKind::Stage => format!("{raw}+stage.{commit}"),
        BuildKind::Release => raw.to_string(),
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
                is_development: self.build_kind.is_development(),
                is_stage: self.build_kind.is_stage(),
                release_url: self.release_url.clone(),
            },
            rust_version: self.rust_version.clone(),
            rust_channel: self.rust_channel.clone(),
            rust_edition: self.rust_edition.clone(),
            build_time: self.build_time,
            git: self.git.clone(),
            map: self.map,
        })
    }

    async fn map_info(&self) -> Result<Map, RepositoryError> {
        Ok(self.map)
    }

    async fn server_info(&self) -> Result<Server, RepositoryError> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "hostname lookup failed, falling back to \"unknown\"");
                "unknown".to_string()
            });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_build_appends_dev_suffix() {
        assert_eq!(
            build_display_version("0.1.0", "abc1234", BuildKind::Dev),
            "0.1.0+dev.abc1234"
        );
    }

    #[test]
    fn stage_build_appends_stage_suffix() {
        assert_eq!(
            build_display_version("0.1.0", "abc1234", BuildKind::Stage),
            "0.1.0+stage.abc1234"
        );
    }

    #[test]
    fn release_build_returns_raw_version() {
        assert_eq!(
            build_display_version("0.1.0", "abc1234", BuildKind::Release),
            "0.1.0"
        );
    }

    #[test]
    fn build_kind_classification_is_disjoint() {
        assert!(BuildKind::Dev.is_development());
        assert!(!BuildKind::Dev.is_stage());

        assert!(!BuildKind::Stage.is_development());
        assert!(BuildKind::Stage.is_stage());

        assert!(!BuildKind::Release.is_development());
        assert!(!BuildKind::Release.is_stage());
    }
}
