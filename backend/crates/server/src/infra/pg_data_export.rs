use std::collections::{BTreeSet, HashMap};

use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    data_export::{
        ExportAuth, ExportEntity, ExportLease, ExportReading, ExportReadingSource, ExportRef,
        ExportRefReader, ExportRefWriter, ExportRunStore, ExportTargetConfig, ExportTargetReader,
        ExportTargetSnapshot, ExportTargetWriter, ReadingBatch, RemoteId, RunResult,
        StoredExportTarget,
    },
    module::ModuleKey,
    organization::Organization,
    sensor_model::{SensorAbilityName, SensorAbilityUnit},
    target::TargetRef,
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
        // A changed base URL is a different server: restart activation and cursor
        // in the INSERT and drop the refs in the same transaction. Every save lifts
        // the backoff, so corrected credentials are retried within one interval.
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
                   failure_count  = 0,
                   next_attempt_at = NULL,
                   last_run_at = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_run_at END,
                   last_success_at = CASE WHEN data_export_targets.base_url <> EXCLUDED.base_url
                                         THEN NULL ELSE data_export_targets.last_success_at END,
                   last_error = NULL,
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
        lease: &ExportLease,
        export_ref: &ExportRef,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO data_export_refs
                   (organization_id, ge_key, entity, remote_id, content_hash, last_value, deleted_marked)
               SELECT $1, $2, $3, $4, $5, $6, $7
               WHERE EXISTS (SELECT 1 FROM data_export_targets
                             WHERE organization_id = $1 AND activated_at = $8
                               AND lease_token = $9
                             FOR SHARE)
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
            lease.generation,
            lease.token,
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
        let ids =
            sqlx::query_scalar!("SELECT organization_id FROM data_export_targets WHERE enabled")
                .fetch_all(&self.pool)
                .await?;
        Ok(ids.into_iter().map(Id::new).collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn due(&self, interval_secs: i64) -> Result<Vec<Id<Organization>>, RepositoryError> {
        let ids = sqlx::query_scalar!(
            r#"SELECT organization_id FROM data_export_targets
               WHERE enabled
                 AND (leased_until IS NULL OR leased_until < now())
                 AND (next_attempt_at IS NULL OR next_attempt_at <= now())
                 AND (last_run_at IS NULL
                      OR last_run_at <= now() - make_interval(secs => $1::bigint::float8))
               ORDER BY organization_id"#,
            interval_secs,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids.into_iter().map(Id::new).collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn try_lease(
        &self,
        org: Id<Organization>,
        token: Uuid,
        lease_secs: i64,
        interval_secs: i64,
    ) -> Result<Option<StoredExportTarget>, RepositoryError> {
        let row = sqlx::query_as!(
            ExportTargetSnapshot,
            r#"UPDATE data_export_targets
               SET leased_until = now() + make_interval(secs => $2::bigint::float8),
                   lease_token = $4
               WHERE organization_id = $1
                 AND enabled
                 AND (leased_until IS NULL OR leased_until < now())
                 AND (next_attempt_at IS NULL OR next_attempt_at <= now())
                 AND (last_run_at IS NULL
                      OR last_run_at <= now() - make_interval(secs => $3::bigint::float8))
               RETURNING organization_id, enabled, base_url, auth_kind, auth_principal,
                         auth_secret, auth_token_url, auth_scope, activated_at,
                         last_run_at, last_success_at, last_error, last_run_stats"#,
            org.value(),
            lease_secs,
            interval_secs,
            token,
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(StoredExportTarget::reconstitute).transpose()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn renew_lease(
        &self,
        org: Id<Organization>,
        lease: &ExportLease,
        lease_secs: i64,
    ) -> Result<bool, RepositoryError> {
        let renewed = sqlx::query!(
            r#"UPDATE data_export_targets
               SET leased_until = now() + make_interval(secs => $4::bigint::float8)
               WHERE organization_id = $1 AND activated_at = $2 AND lease_token = $3"#,
            org.value(),
            lease.generation,
            lease.token,
            lease_secs,
        )
        .execute(&self.pool)
        .await?;
        Ok(renewed.rows_affected() == 1)
    }

    #[tracing::instrument(level = "debug", skip(self, result))]
    async fn finish(
        &self,
        org: Id<Organization>,
        lease: &ExportLease,
        result: &RunResult,
    ) -> Result<(), RepositoryError> {
        match result {
            RunResult::Succeeded(stats) => {
                let stats = serde_json::to_value(stats)
                    .map_err(|e| RepositoryError::Internal(format!("run stats: {e}")))?;
                sqlx::query!(
                    r#"UPDATE data_export_targets
                       SET leased_until = NULL, lease_token = NULL, last_run_at = now(),
                           last_success_at = now(), last_error = NULL, last_run_stats = $2,
                           failure_count = 0, next_attempt_at = NULL
                       WHERE organization_id = $1 AND activated_at = $3 AND lease_token = $4"#,
                    org.value(),
                    stats,
                    lease.generation,
                    lease.token,
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
                       SET leased_until = NULL, lease_token = NULL, last_run_at = now(),
                           last_error = $2,
                           failure_count = failure_count + 1,
                           next_attempt_at = now() + make_interval(
                               secs => LEAST($3::bigint * power(2, LEAST(failure_count + 1, 20)), 3600)::float8)
                       WHERE organization_id = $1 AND activated_at = $4 AND lease_token = $5"#,
                    org.value(),
                    error,
                    base_interval_secs,
                    lease.generation,
                    lease.token,
                )
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn cursor(&self, org: Id<Organization>) -> Result<Option<Uuid>, RepositoryError> {
        Ok(sqlx::query_scalar!(
            "SELECT cursor_reading_id FROM data_export_targets WHERE organization_id = $1",
            org.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn advance_cursor(
        &self,
        org: Id<Organization>,
        lease: &ExportLease,
        cursor: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"UPDATE data_export_targets SET cursor_reading_id = $3
               WHERE organization_id = $1 AND activated_at = $2 AND lease_token = $4
                 AND (cursor_reading_id IS NULL OR cursor_reading_id < $3)"#,
            org.value(),
            lease.generation,
            cursor,
            lease.token,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ExportReadingSource for PgDataExportRepository {
    #[tracing::instrument(level = "debug", skip(self, active), fields(pairs = active.len()))]
    async fn readings_after(
        &self,
        active: &[(ModuleKey, Id<Organization>)],
        after: Uuid,
        before: Uuid,
        limit: i64,
    ) -> Result<ReadingBatch, RepositoryError> {
        let modules: Vec<String> = active.iter().map(|(m, _)| m.as_str().to_string()).collect();
        let orgs: Vec<Uuid> = active.iter().map(|(_, o)| o.value()).collect();
        let rows = sqlx::query!(
            r#"WITH active AS (
                   SELECT DISTINCT * FROM unnest($1::text[], $2::uuid[]) AS a(module_key, organization_id)
               ),
               batch AS (
                   SELECT d.id, d.sensor_id, a.module_key, a.target_kind, a.target_id, m.name AS model
                   FROM sensor_data d
                   JOIN sensors s ON s.id = d.sensor_id
                   JOIN sensor_attachments a ON a.sensor_id = d.sensor_id
                   JOIN active ON active.module_key = a.module_key AND active.organization_id = s.organization_id
                   JOIN sensor_models m ON m.id = s.model_id
                   WHERE d.id > $3 AND d.id < $5
                   ORDER BY d.id
                   LIMIT $4
               )
               SELECT b.id AS "reading_id!", b.sensor_id AS "sensor_id!", b.module_key AS "module_key!",
                      b.target_kind AS "target_kind!", b.target_id AS "target_id!", b.model AS "model!",
                      ab.ability AS "ability?", ab.unit AS "unit?: SensorAbilityUnit", ma.depth_cm AS "depth_cm?",
                      v.value::float8 AS "value?", v.plausible AS "plausible?", v.quality_reason
               FROM batch b
               LEFT JOIN sensor_data_ability_values v ON v.sensor_data_id = b.id
               LEFT JOIN sensor_model_abilities ma ON ma.id = v.sensor_model_ability_id
               LEFT JOIN sensor_abilities ab ON ab.id = ma.sensor_ability_id
               ORDER BY b.id, ab.ability, ma.depth_cm"#,
            &modules,
            &orgs,
            after,
            limit,
            before,
        )
        .fetch_all(&self.pool)
        .await?;
        let scanned_until = rows.iter().map(|r| r.reading_id).max();
        let scanned = rows
            .iter()
            .map(|r| r.reading_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len() as i64;
        let rows = rows
            .into_iter()
            .filter_map(|r| {
                let (ability, unit, depth_cm, value, plausible) = (
                    r.ability.as_deref()?,
                    r.unit?,
                    r.depth_cm?,
                    r.value?,
                    r.plausible?,
                );
                let broken = |what: String| {
                    RepositoryError::DataIntegrity(format!(
                        "export reading {}: {what}",
                        r.reading_id
                    ))
                };
                Some((|| {
                    Ok(ExportReading {
                        reading_id: r.reading_id,
                        sensor_id: r.sensor_id.clone(),
                        sensor_model: r.model.clone(),
                        target: TargetRef::new(&r.module_key, &r.target_kind, r.target_id)
                            .map_err(|e| broken(e.to_string()))?,
                        ability: SensorAbilityName::from_str(ability)
                            .map_err(|e| broken(e.to_string()))?,
                        unit,
                        depth_cm,
                        value,
                        quality_issue: (!plausible).then(|| {
                            r.quality_reason
                                .clone()
                                .unwrap_or_else(|| "implausible".into())
                        }),
                        measured_at: domain::uuid_v7_timestamp(&r.reading_id)
                            .ok_or_else(|| broken("not a UUID v7".into()))?,
                    })
                })())
            })
            .collect::<Result<Vec<_>, RepositoryError>>()?;
        Ok(ReadingBatch {
            rows,
            scanned_until,
            scanned,
        })
    }
}
