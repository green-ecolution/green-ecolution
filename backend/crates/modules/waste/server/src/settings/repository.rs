//! Uncached for the same reasons as the core's settings adapter: small tables
//! and no stale values on other replicas.

use std::{collections::HashMap, sync::Arc};

use domain::{
    Id, RepositoryError,
    organization::{Organization, OrganizationReader},
};
use sqlx::PgPool;
use uuid::Uuid;
use waste_domain::settings::FullThreshold;

use super::{
    EffectiveWasteSettings, WasteDefaults, WasteLevel, WasteResolution, WasteSettings,
    WasteSettingsResolver, WasteSettingsUpdate, resolve,
};
use server::http::v1::dto::settings::ChangeStamp;

pub const FULL_THRESHOLD_KEY: &str = "full_threshold";

pub struct PgWasteSettings {
    pool: PgPool,
    defaults: WasteDefaults,
    org_reader: Arc<dyn OrganizationReader>,
}

impl PgWasteSettings {
    pub fn new(
        pool: PgPool,
        defaults: WasteDefaults,
        org_reader: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            pool,
            defaults,
            org_reader,
        }
    }

    async fn chain(&self, target: Id<Organization>) -> Result<Vec<WasteLevel>, RepositoryError> {
        let hierarchy = self.org_reader.hierarchy().await?;
        if !hierarchy.knows(target) {
            return Err(RepositoryError::NotFound);
        }
        let ancestors = hierarchy.ancestors_from_root(target);
        let ids: Vec<Uuid> = ancestors.iter().map(|id| id.value()).collect();

        let rows = sqlx::query!(
            r#"SELECT o.id AS organization_id,
                      w.full_threshold_percent AS "full_threshold_percent?",
                      COALESCE(s.descendants_may_override, TRUE) AS "descendants_may_override!"
               FROM organizations o
               LEFT JOIN waste_settings w ON w.organization_id = o.id
               LEFT JOIN organization_settings s ON s.organization_id = o.id
               WHERE o.id = ANY($1)"#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_org = HashMap::new();
        for row in rows {
            let org = Id::new(row.organization_id);
            let level = WasteLevel {
                settings: WasteSettings {
                    organization_id: org,
                    full_threshold: row
                        .full_threshold_percent
                        .map(|p| FullThreshold::new(i64::from(p)))
                        .transpose()?,
                },
                descendants_may_override: row.descendants_may_override,
            };
            by_org.insert(org, level);
        }

        Ok(ancestors
            .into_iter()
            .map(|id| {
                by_org.get(&id).copied().unwrap_or(WasteLevel {
                    settings: WasteSettings::empty(id),
                    descendants_may_override: true,
                })
            })
            .collect())
    }

    pub async fn own(&self, org: Id<Organization>) -> Result<WasteSettings, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT full_threshold_percent
               FROM waste_settings WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_optional(&self.pool)
        .await?;
        stored(org, row.map(|r| r.full_threshold_percent))
    }

    pub async fn resolution(
        &self,
        org: Id<Organization>,
    ) -> Result<WasteResolution, RepositoryError> {
        let chain = self.chain(org).await?;
        Ok(resolve(&self.defaults, &chain, org))
    }

    pub async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<&'static str, ChangeStamp>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT changed_at, changed_by
               FROM organization_settings_history
               WHERE organization_id = $1 AND setting_key = 'full_threshold'
               ORDER BY changed_at DESC, id DESC
               LIMIT 1"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    FULL_THRESHOLD_KEY,
                    ChangeStamp {
                        changed_at: row.changed_at,
                        changed_by: row.changed_by,
                    },
                )
            })
            .collect())
    }

    pub async fn apply(
        &self,
        org: Id<Organization>,
        update: WasteSettingsUpdate,
        actor: Option<Uuid>,
    ) -> Result<WasteSettings, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"SELECT full_threshold_percent
               FROM waste_settings WHERE organization_id = $1 FOR UPDATE"#,
            org.value()
        )
        .fetch_optional(&mut *tx)
        .await?;
        let current = stored(org, row.map(|r| r.full_threshold_percent))?;

        let next = WasteSettings {
            organization_id: org,
            full_threshold: update.full_threshold.resolve(current.full_threshold),
        };

        sqlx::query!(
            r#"INSERT INTO waste_settings (organization_id, full_threshold_percent)
               VALUES ($1, $2)
               ON CONFLICT (organization_id) DO UPDATE SET
                   full_threshold_percent = EXCLUDED.full_threshold_percent"#,
            org.value(),
            next.full_threshold.map(|v| i16::from(v.percent())),
        )
        .execute(&mut *tx)
        .await?;

        let previous = current
            .full_threshold
            .map(|v| serde_json::json!(v.percent()));
        let new = next.full_threshold.map(|v| serde_json::json!(v.percent()));
        if previous != new {
            sqlx::query!(
                r#"INSERT INTO organization_settings_history
                       (id, organization_id, setting_key, previous_value, new_value, changed_by)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
                Uuid::now_v7(),
                org.value(),
                FULL_THRESHOLD_KEY,
                previous,
                new,
                actor,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(next)
    }
}

fn stored(
    org: Id<Organization>,
    row: Option<Option<i16>>,
) -> Result<WasteSettings, RepositoryError> {
    let Some(percent) = row else {
        return Ok(WasteSettings::empty(org));
    };
    Ok(WasteSettings {
        organization_id: org,
        full_threshold: percent
            .map(|p| FullThreshold::new(i64::from(p)))
            .transpose()?,
    })
}

#[async_trait::async_trait]
impl WasteSettingsResolver for PgWasteSettings {
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveWasteSettings, RepositoryError> {
        Ok(self.resolution(org).await?.effective)
    }
}
