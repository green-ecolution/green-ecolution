//! Uncached for the same reasons as the core's settings adapter: small tables
//! and no stale values on other replicas.

use std::{collections::HashMap, sync::Arc};

use domain::{
    Id, RepositoryError,
    organization::{Organization, OrganizationReader},
};
use irrigation_domain::settings::{JustWateredTtl, WaterDemand};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    EffectiveIrrigationSettings, IrrigationDefaults, IrrigationLevel, IrrigationResolution,
    IrrigationSettings, IrrigationSettingsResolver, IrrigationSettingsUpdate, resolve,
};
use server::http::v1::dto::settings::ChangeStamp;

const WATER_DEMAND_KEY: &str = "water_demand";
const JUST_WATERED_TTL_KEY: &str = "just_watered_ttl";

pub struct PgIrrigationSettings {
    pool: PgPool,
    defaults: IrrigationDefaults,
    org_reader: Arc<dyn OrganizationReader>,
}

impl PgIrrigationSettings {
    pub fn new(
        pool: PgPool,
        defaults: IrrigationDefaults,
        org_reader: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            pool,
            defaults,
            org_reader,
        }
    }

    async fn chain(
        &self,
        target: Id<Organization>,
    ) -> Result<Vec<IrrigationLevel>, RepositoryError> {
        let hierarchy = self.org_reader.hierarchy().await?;
        if !hierarchy.knows(target) {
            return Err(RepositoryError::NotFound);
        }
        let ancestors = hierarchy.ancestors_from_root(target);
        let ids: Vec<Uuid> = ancestors.iter().map(|id| id.value()).collect();

        let rows = sqlx::query!(
            r#"SELECT o.id AS organization_id,
                      i.water_demand_liters AS "water_demand_liters?",
                      i.just_watered_ttl_secs AS "just_watered_ttl_secs?",
                      COALESCE(s.descendants_may_override, TRUE) AS "descendants_may_override!"
               FROM organizations o
               LEFT JOIN irrigation_settings i ON i.organization_id = o.id
               LEFT JOIN organization_settings s ON s.organization_id = o.id
               WHERE o.id = ANY($1)"#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_org = HashMap::new();
        for row in rows {
            let org = Id::new(row.organization_id);
            let level = IrrigationLevel {
                settings: IrrigationSettings {
                    organization_id: org,
                    water_demand: row.water_demand_liters.map(WaterDemand::new).transpose()?,
                    just_watered_ttl: row
                        .just_watered_ttl_secs
                        .map(JustWateredTtl::new)
                        .transpose()?,
                },
                descendants_may_override: row.descendants_may_override,
            };
            by_org.insert(org, level);
        }

        Ok(ancestors
            .into_iter()
            .map(|id| {
                by_org.get(&id).copied().unwrap_or(IrrigationLevel {
                    settings: IrrigationSettings::empty(id),
                    descendants_may_override: true,
                })
            })
            .collect())
    }

    pub async fn own(&self, org: Id<Organization>) -> Result<IrrigationSettings, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT water_demand_liters, just_watered_ttl_secs
               FROM irrigation_settings WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_optional(&self.pool)
        .await?;
        stored(
            org,
            row.map(|r| (r.water_demand_liters, r.just_watered_ttl_secs)),
        )
    }

    pub async fn resolution(
        &self,
        org: Id<Organization>,
    ) -> Result<IrrigationResolution, RepositoryError> {
        let chain = self.chain(org).await?;
        Ok(resolve(&self.defaults, &chain, org))
    }

    pub async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<&'static str, ChangeStamp>, RepositoryError> {
        // `changed_at` is transaction-start time, so `id` (uuid v7) breaks ties
        // between keys written by one update.
        let rows = sqlx::query!(
            r#"SELECT DISTINCT ON (setting_key)
                      setting_key AS "setting_key!", changed_at, changed_by
               FROM organization_settings_history
               WHERE organization_id = $1
                 AND setting_key IN ('water_demand', 'just_watered_ttl')
               ORDER BY setting_key, changed_at DESC, id DESC"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = HashMap::new();
        for row in rows {
            let key = match row.setting_key.as_str() {
                WATER_DEMAND_KEY => WATER_DEMAND_KEY,
                JUST_WATERED_TTL_KEY => JUST_WATERED_TTL_KEY,
                _ => continue,
            };
            out.insert(
                key,
                ChangeStamp {
                    changed_at: row.changed_at,
                    changed_by: row.changed_by,
                },
            );
        }
        Ok(out)
    }

    pub async fn apply(
        &self,
        org: Id<Organization>,
        update: IrrigationSettingsUpdate,
        actor: Option<Uuid>,
    ) -> Result<IrrigationSettings, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query!(
            r#"SELECT water_demand_liters, just_watered_ttl_secs
               FROM irrigation_settings WHERE organization_id = $1 FOR UPDATE"#,
            org.value()
        )
        .fetch_optional(&mut *tx)
        .await?;
        let current = stored(
            org,
            row.map(|r| (r.water_demand_liters, r.just_watered_ttl_secs)),
        )?;

        let next = IrrigationSettings {
            organization_id: org,
            water_demand: update.water_demand.resolve(current.water_demand),
            just_watered_ttl: update.just_watered_ttl.resolve(current.just_watered_ttl),
        };

        sqlx::query!(
            r#"INSERT INTO irrigation_settings
                   (organization_id, water_demand_liters, just_watered_ttl_secs)
               VALUES ($1, $2, $3)
               ON CONFLICT (organization_id) DO UPDATE SET
                   water_demand_liters = EXCLUDED.water_demand_liters,
                   just_watered_ttl_secs = EXCLUDED.just_watered_ttl_secs"#,
            org.value(),
            next.water_demand.map(|v| v.liters()),
            next.just_watered_ttl.map(|v| v.seconds()),
        )
        .execute(&mut *tx)
        .await?;

        let changes = [
            (
                WATER_DEMAND_KEY,
                current.water_demand.map(|v| serde_json::json!(v.liters())),
                next.water_demand.map(|v| serde_json::json!(v.liters())),
            ),
            (
                JUST_WATERED_TTL_KEY,
                current
                    .just_watered_ttl
                    .map(|v| serde_json::json!(v.seconds())),
                next.just_watered_ttl
                    .map(|v| serde_json::json!(v.seconds())),
            ),
        ];
        for (key, previous, new) in changes {
            if previous == new {
                continue;
            }
            sqlx::query!(
                r#"INSERT INTO organization_settings_history
                       (id, organization_id, setting_key, previous_value, new_value, changed_by)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
                Uuid::now_v7(),
                org.value(),
                key,
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
    row: Option<(Option<f64>, Option<i64>)>,
) -> Result<IrrigationSettings, RepositoryError> {
    let Some((liters, secs)) = row else {
        return Ok(IrrigationSettings::empty(org));
    };
    Ok(IrrigationSettings {
        organization_id: org,
        water_demand: liters.map(WaterDemand::new).transpose()?,
        just_watered_ttl: secs.map(JustWateredTtl::new).transpose()?,
    })
}

#[async_trait::async_trait]
impl IrrigationSettingsResolver for PgIrrigationSettings {
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveIrrigationSettings, RepositoryError> {
        Ok(self.resolution(org).await?.effective)
    }
}
