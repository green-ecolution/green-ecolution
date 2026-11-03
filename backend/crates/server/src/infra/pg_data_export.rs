use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    data_export::{
        ExportAuth, ExportTargetConfig, ExportTargetReader, ExportTargetSnapshot,
        ExportTargetWriter, StoredExportTarget,
    },
    organization::Organization,
};

pub struct PgDataExportRepository {
    pool: PgPool,
}

impl PgDataExportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct AuthColumns<'a> {
    kind: &'static str,
    principal: Option<&'a str>,
    secret: Option<&'a str>,
    token_url: Option<&'a str>,
    scope: Option<&'a str>,
}

fn auth_columns(auth: &ExportAuth) -> AuthColumns<'_> {
    let scope = match auth {
        ExportAuth::OAuth2ClientCredentials { scope, .. } => scope.as_ref().map(|s| s.as_str()),
        _ => None,
    };
    AuthColumns {
        kind: auth.kind().as_str(),
        principal: auth.principal().map(|p| p.as_str()),
        secret: auth.secret().map(|s| s.expose()),
        token_url: auth.token_url().map(|u| u.as_str()),
        scope,
    }
}

#[async_trait::async_trait]
impl ExportTargetReader for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn find(
        &self,
        org: Id<Organization>,
    ) -> Result<Option<StoredExportTarget>, RepositoryError> {
        let row = sqlx::query_as!(
            ExportTargetSnapshot,
            r#"SELECT organization_id, enabled, base_url, auth_kind, auth_principal,
                      auth_secret, auth_token_url, auth_scope, activated_at,
                      last_run_at, last_success_at, last_error, last_run_stats
               FROM data_export_targets
               WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(StoredExportTarget::reconstitute).transpose()
    }
}

#[async_trait::async_trait]
impl ExportTargetWriter for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self, config))]
    async fn upsert(
        &self,
        org: Id<Organization>,
        config: &ExportTargetConfig,
    ) -> Result<StoredExportTarget, RepositoryError> {
        let auth = auth_columns(&config.auth);
        // A changed base URL is a different server: restart activation, cursor
        // and backoff in the same statement so no pass sees a half-reset row.
        let row = sqlx::query_as!(
            ExportTargetSnapshot,
            r#"INSERT INTO data_export_targets
                   (organization_id, enabled, base_url, auth_kind, auth_principal,
                    auth_secret, auth_token_url, auth_scope)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (organization_id) DO UPDATE SET
                   enabled        = EXCLUDED.enabled,
                   base_url       = EXCLUDED.base_url,
                   auth_kind      = EXCLUDED.auth_kind,
                   auth_principal = EXCLUDED.auth_principal,
                   auth_secret    = EXCLUDED.auth_secret,
                   auth_token_url = EXCLUDED.auth_token_url,
                   auth_scope     = EXCLUDED.auth_scope,
                   activated_at   = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN now() ELSE data_export_targets.activated_at END,
                   cursor_reading_id = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.cursor_reading_id END,
                   failure_count  = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN 0 ELSE data_export_targets.failure_count END,
                   next_attempt_at = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.next_attempt_at END
               RETURNING organization_id, enabled, base_url, auth_kind, auth_principal,
                         auth_secret, auth_token_url, auth_scope, activated_at,
                         last_run_at, last_success_at, last_error, last_run_stats"#,
            org.value(),
            config.enabled,
            config.base_url.as_str(),
            auth.kind,
            auth.principal,
            auth.secret,
            auth.token_url,
            auth.scope,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match RepositoryError::from(e) {
            RepositoryError::ForeignKeyViolation(_) => RepositoryError::NotFound,
            other => other,
        })?;
        StoredExportTarget::reconstitute(row)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn delete(&self, org: Id<Organization>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            "DELETE FROM data_export_targets WHERE organization_id = $1",
            org.value()
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
