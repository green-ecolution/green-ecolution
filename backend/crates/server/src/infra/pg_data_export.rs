use std::collections::{BTreeSet, HashMap};

use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    data_export::{
        ExportAuth, ExportEntity, ExportRef, ExportRefReader, ExportRefWriter, ExportRunStore,
        ExportTargetConfig, ExportTargetReader, ExportTargetSnapshot, ExportTargetWriter, RemoteId,
        RunResult, StoredExportTarget,
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
        let mut tx = self.pool.begin().await?;
        let previous: Option<String> = sqlx::query_scalar!(
            "SELECT base_url FROM data_export_targets WHERE organization_id = $1 FOR UPDATE",
            org.value()
        )
        .fetch_optional(&mut *tx)
        .await?;
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
                                         THEN NULL ELSE data_export_targets.next_attempt_at END,
                   last_run_at = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_run_at END,
                   last_success_at = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_success_at END,
                   last_error = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_error END,
                   last_run_stats = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_run_stats END
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
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match RepositoryError::from(e) {
            RepositoryError::ForeignKeyViolation(_) => RepositoryError::NotFound,
            other => other,
        })?;
        if previous.is_some_and(|url| url != config.base_url.as_str()) {
            sqlx::query!(
                "DELETE FROM data_export_refs WHERE organization_id = $1",
                org.value()
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
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

#[async_trait::async_trait]
impl ExportRefReader for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn refs(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<String, ExportRef>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT ge_key, entity, remote_id, content_hash, last_value, deleted_marked
               FROM data_export_refs WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|r| {
                let entity = ExportEntity::parse(&r.entity).ok_or_else(|| {
                    RepositoryError::DataIntegrity(format!(
                        "data_export_refs: unknown entity {}",
                        r.entity
                    ))
                })?;
                Ok((
                    r.ge_key.clone(),
                    ExportRef {
                        ge_key: r.ge_key,
                        entity,
                        remote_id: RemoteId(r.remote_id),
                        content_hash: r.content_hash,
                        last_value: r.last_value,
                        deleted_marked: r.deleted_marked,
                    },
                ))
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl ExportRefWriter for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self, export_ref), fields(ge_key = %export_ref.ge_key))]
    async fn put_ref(
        &self,
        org: Id<Organization>,
        export_ref: &ExportRef,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO data_export_refs
                   (organization_id, ge_key, entity, remote_id, content_hash, last_value, deleted_marked)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (organization_id, ge_key) DO UPDATE SET
                   entity = EXCLUDED.entity,
                   remote_id = EXCLUDED.remote_id,
                   content_hash = EXCLUDED.content_hash,
                   last_value = EXCLUDED.last_value,
                   deleted_marked = EXCLUDED.deleted_marked"#,
            org.value(),
            export_ref.ge_key,
            export_ref.entity.as_str(),
            export_ref.remote_id.as_json(),
            export_ref.content_hash,
            export_ref.last_value,
            export_ref.deleted_marked,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ExportRunStore for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn target_orgs(&self) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        let ids = sqlx::query_scalar!("SELECT organization_id FROM data_export_targets")
            .fetch_all(&self.pool)
            .await?;
        Ok(ids.into_iter().map(Id::new).collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn due(&self) -> Result<Vec<Id<Organization>>, RepositoryError> {
        let ids = sqlx::query_scalar!(
            r#"SELECT organization_id FROM data_export_targets
               WHERE enabled
                 AND (leased_until IS NULL OR leased_until < now())
                 AND (next_attempt_at IS NULL OR next_attempt_at <= now())
               ORDER BY organization_id"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids.into_iter().map(Id::new).collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_lease(
        &self,
        org: Id<Organization>,
        lease_secs: i64,
    ) -> Result<Option<StoredExportTarget>, RepositoryError> {
        let row = sqlx::query_as!(
            ExportTargetSnapshot,
            r#"UPDATE data_export_targets
               SET leased_until = now() + make_interval(secs => $2::bigint::float8)
               WHERE organization_id = $1
                 AND enabled
                 AND (leased_until IS NULL OR leased_until < now())
                 AND (next_attempt_at IS NULL OR next_attempt_at <= now())
               RETURNING organization_id, enabled, base_url, auth_kind, auth_principal,
                         auth_secret, auth_token_url, auth_scope, activated_at,
                         last_run_at, last_success_at, last_error, last_run_stats"#,
            org.value(),
            lease_secs,
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(StoredExportTarget::reconstitute).transpose()
    }

    #[tracing::instrument(level = "debug", skip(self, result))]
    async fn finish(
        &self,
        org: Id<Organization>,
        result: &RunResult,
    ) -> Result<(), RepositoryError> {
        match result {
            RunResult::Succeeded(stats) => {
                let stats = serde_json::to_value(stats)
                    .map_err(|e| RepositoryError::Internal(format!("run stats: {e}")))?;
                sqlx::query!(
                    r#"UPDATE data_export_targets
                       SET leased_until = NULL, last_run_at = now(), last_success_at = now(),
                           last_error = NULL, last_run_stats = $2,
                           failure_count = 0, next_attempt_at = NULL
                       WHERE organization_id = $1"#,
                    org.value(),
                    stats,
                )
                .execute(&self.pool)
                .await?;
            }
            RunResult::Failed {
                error,
                base_interval_secs,
            } => {
                let error: String = error.chars().take(2000).collect();
                // The right-hand `failure_count` is the old value, so the first
                // failure waits interval x 2.
                sqlx::query!(
                    r#"UPDATE data_export_targets
                       SET leased_until = NULL, last_run_at = now(), last_error = $2,
                           failure_count = failure_count + 1,
                           next_attempt_at = now() + make_interval(
                               secs => LEAST($3::bigint * power(2, failure_count + 1), 3600)::float8)
                       WHERE organization_id = $1"#,
                    org.value(),
                    error,
                    base_interval_secs,
                )
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }
}
