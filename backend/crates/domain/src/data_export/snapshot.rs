use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    ExportAuth, ExportAuthKind, ExportPrincipal, ExportRunStats, ExportScope, ExportSecret,
    ExportTarget, ExportTargetConfig, ExportTargetStatus, ExportUrl, StoredExportTarget,
};
use crate::{Id, RepositoryError};

#[doc(hidden)]
pub struct ExportTargetSnapshot {
    pub organization_id: Uuid,
    pub enabled: bool,
    pub base_url: String,
    pub auth_kind: String,
    pub auth_principal: Option<String>,
    pub auth_secret: Option<String>,
    pub auth_token_url: Option<String>,
    pub auth_scope: Option<String>,
    pub activated_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub last_run_stats: Option<serde_json::Value>,
}

impl std::fmt::Debug for ExportTargetSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportTargetSnapshot")
            .field("organization_id", &self.organization_id)
            .field("auth_kind", &self.auth_kind)
            .finish_non_exhaustive()
    }
}

impl StoredExportTarget {
    /// The table's CHECK constraint guarantees the auth shape; a violation here
    /// means the row was written around it.
    pub fn reconstitute(row: ExportTargetSnapshot) -> Result<Self, RepositoryError> {
        let broken =
            |what: &str| RepositoryError::DataIntegrity(format!("data_export_targets: {what}"));
        let principal = || {
            row.auth_principal
                .clone()
                .map(ExportPrincipal::reconstitute)
                .ok_or_else(|| broken("principal missing"))
        };
        let secret = || {
            row.auth_secret
                .clone()
                .map(ExportSecret::reconstitute)
                .ok_or_else(|| broken("secret missing"))
        };
        let auth = match ExportAuthKind::parse(&row.auth_kind) {
            Some(ExportAuthKind::None) => ExportAuth::None,
            Some(ExportAuthKind::Basic) => ExportAuth::Basic {
                username: principal()?,
                password: secret()?,
            },
            Some(ExportAuthKind::OAuth2ClientCredentials) => ExportAuth::OAuth2ClientCredentials {
                token_url: ExportUrl::reconstitute(
                    row.auth_token_url
                        .as_deref()
                        .ok_or_else(|| broken("token url missing"))?,
                )?,
                client_id: principal()?,
                client_secret: secret()?,
                scope: row.auth_scope.clone().map(ExportScope::reconstitute),
            },
            None => return Err(broken(&format!("unknown auth kind {}", row.auth_kind))),
        };
        let last_run_stats = row
            .last_run_stats
            .map(serde_json::from_value::<ExportRunStats>)
            .transpose()
            .map_err(|e| broken(&format!("last_run_stats: {e}")))?;
        Ok(Self {
            target: ExportTarget {
                organization_id: Id::new(row.organization_id),
                config: ExportTargetConfig {
                    enabled: row.enabled,
                    base_url: ExportUrl::reconstitute(&row.base_url)?,
                    auth,
                },
                activated_at: row.activated_at,
            },
            status: ExportTargetStatus {
                last_run_at: row.last_run_at,
                last_success_at: row.last_success_at,
                last_error: row.last_error,
                last_run_stats,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_export::{ExportAuthKind, ExportTargetView};
    use chrono::Utc;
    use uuid::Uuid;

    fn row(kind: &str) -> ExportTargetSnapshot {
        ExportTargetSnapshot {
            organization_id: Uuid::now_v7(),
            enabled: true,
            base_url: "https://frost.example.org/FROST-Server/v1.1".into(),
            auth_kind: kind.into(),
            auth_principal: Some("ge".into()),
            auth_secret: Some("pw".into()),
            auth_token_url: None,
            auth_scope: None,
            activated_at: Utc::now(),
            last_run_at: None,
            last_success_at: None,
            last_error: None,
            last_run_stats: None,
        }
    }

    #[test]
    fn rebuilds_a_basic_target_and_its_view_hides_the_secret() {
        let stored = StoredExportTarget::reconstitute(row("basic")).unwrap();
        let view = ExportTargetView::from(&stored);
        assert_eq!(view.auth.kind, ExportAuthKind::Basic);
        assert_eq!(view.auth.principal.as_deref(), Some("ge"));
        assert!(view.auth.has_secret);
    }

    #[test]
    fn an_unknown_kind_is_a_data_integrity_error() {
        assert!(matches!(
            StoredExportTarget::reconstitute(row("bearer")),
            Err(crate::RepositoryError::DataIntegrity(_))
        ));
    }

    #[test]
    fn a_missing_secret_for_basic_is_a_data_integrity_error() {
        let mut r = row("basic");
        r.auth_secret = None;
        assert!(matches!(
            StoredExportTarget::reconstitute(r),
            Err(crate::RepositoryError::DataIntegrity(_))
        ));
    }
}
