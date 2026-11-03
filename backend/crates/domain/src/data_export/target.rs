use chrono::{DateTime, Utc};

use super::{ExportAuth, ExportAuthDraft, ExportAuthKind, ExportUrl};
use crate::{Id, organization::Organization};

#[derive(Debug, Clone)]
pub struct ExportTargetDraft {
    pub enabled: bool,
    pub base_url: ExportUrl,
    pub auth: ExportAuthDraft,
}

#[derive(Debug, Clone)]
pub struct ExportTargetConfig {
    pub enabled: bool,
    pub base_url: ExportUrl,
    pub auth: ExportAuth,
}

#[derive(Debug, Clone)]
pub struct ExportTarget {
    pub organization_id: Id<Organization>,
    pub config: ExportTargetConfig,
    /// Readings from before this instant are not exported (no backfill).
    pub activated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExportRunStats {
    pub things_written: u32,
    pub observations_written: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExportTargetStatus {
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub last_run_stats: Option<ExportRunStats>,
}

#[derive(Debug, Clone)]
pub struct StoredExportTarget {
    pub target: ExportTarget,
    pub status: ExportTargetStatus,
}

/// What an API caller may see of a target: everything but the secret.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportAuthSummary {
    pub kind: ExportAuthKind,
    pub principal: Option<String>,
    pub token_url: Option<String>,
    pub scope: Option<String>,
    pub has_secret: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportTargetView {
    pub organization_id: Id<Organization>,
    pub enabled: bool,
    pub base_url: String,
    pub auth: ExportAuthSummary,
    pub activated_at: DateTime<Utc>,
    pub status: ExportTargetStatus,
}

impl From<&StoredExportTarget> for ExportTargetView {
    fn from(stored: &StoredExportTarget) -> Self {
        let config = &stored.target.config;
        let scope = match &config.auth {
            ExportAuth::OAuth2ClientCredentials { scope, .. } => {
                scope.as_ref().map(|s| s.as_str().to_string())
            }
            _ => None,
        };
        Self {
            organization_id: stored.target.organization_id,
            enabled: config.enabled,
            base_url: config.base_url.as_str().to_string(),
            auth: ExportAuthSummary {
                kind: config.auth.kind(),
                principal: config.auth.principal().map(|p| p.as_str().to_string()),
                token_url: config.auth.token_url().map(|u| u.as_str().to_string()),
                scope,
                has_secret: config.auth.secret().is_some(),
            },
            activated_at: stored.target.activated_at,
            status: stored.status.clone(),
        }
    }
}
