//! Nothing here is cached. Both tables are small, the same request already
//! reads the whole organization tree uncached to walk it, and a per-process
//! cache would keep serving pre-write values on every other replica.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    organization::{Organization, OrganizationReader},
    settings::{
        EffectiveSettings, InstanceDefaults, OrganizationSettings, OrganizationSettingsSnapshot,
        Resolution, SettingChangeEntry, SettingKey, SettingValue, SettingsReader, SettingsResolver,
        SettingsUpdate, SettingsWriter,
        values::{
            DefectStreak, JustWateredTtl, MapBounds, MapView, SensorOfflineAfter, WaterDemand,
            ZoomLevel,
        },
    },
    shared::{coordinates::Coordinate, geo::BoundingBox},
};

type SettingsByOrg = HashMap<Id<Organization>, OrganizationSettings>;

pub struct PgSettingsRepository {
    pool: PgPool,
    defaults: InstanceDefaults,
    org_reader: Arc<dyn OrganizationReader>,
}

impl PgSettingsRepository {
    pub fn new(
        pool: PgPool,
        defaults: InstanceDefaults,
        org_reader: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            pool,
            defaults,
            org_reader,
        }
    }

    async fn by_org(&self) -> Result<SettingsByOrg, RepositoryError> {
        let rows = sqlx::query_as!(
            OrganizationSettingsSnapshot,
            r#"SELECT organization_id, water_demand_liters, just_watered_ttl_secs,
                      sensor_offline_after_secs, defect_streak,
                      map_center_lat, map_center_lng,
                      map_bbox_sw_lat, map_bbox_sw_lng,
                      map_bbox_ne_lat, map_bbox_ne_lng,
                      map_min_zoom, map_max_zoom,
                      descendants_may_override
               FROM organization_settings"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_org = SettingsByOrg::new();
        for row in rows {
            let settings = reconstitute(row)?;
            by_org.insert(settings.organization_id, settings);
        }

        Ok(by_org)
    }

    async fn resolve_for(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError> {
        let by_org = self.by_org().await?;
        let hierarchy = self.org_reader.hierarchy().await?;
        // Without this an unknown id resolves to a one-element chain and comes
        // back as a full set of instance defaults, which reads like a real
        // answer. The tree is already loaded, so the check costs nothing.
        if !hierarchy.knows(org) {
            return Err(RepositoryError::NotFound);
        }
        let chain: Vec<OrganizationSettings> = hierarchy
            .ancestors_from_root(org)
            .into_iter()
            .map(|id| {
                by_org
                    .get(&id)
                    .copied()
                    .unwrap_or_else(|| OrganizationSettings::empty(id))
            })
            .collect();
        Ok(domain::settings::resolve(&self.defaults, &chain, org))
    }
}

/// Zoom is a SMALLINT because Postgres has no unsigned type; the column's own
/// check keeps it inside the domain's range, so this only narrows the type.
fn zoom_from_db(level: i16) -> Result<u8, RepositoryError> {
    u8::try_from(level).map_err(|_| {
        RepositoryError::DataIntegrity(format!("stored zoom level {level} is out of range"))
    })
}

/// Rehydration may fail only where the stored row breaks a domain rule the
/// schema cannot express; that is a data-integrity fault, not a not-found.
fn reconstitute(
    snap: OrganizationSettingsSnapshot,
) -> Result<OrganizationSettings, RepositoryError> {
    // The six limit columns are set or absent as a whole; the table's check
    // constraint keeps a half-set group out, so a partial one is impossible.
    let bounds = match (
        snap.map_bbox_sw_lat,
        snap.map_bbox_sw_lng,
        snap.map_bbox_ne_lat,
        snap.map_bbox_ne_lng,
        snap.map_min_zoom,
        snap.map_max_zoom,
    ) {
        (Some(s_lat), Some(s_lng), Some(n_lat), Some(n_lng), Some(min), Some(max)) => {
            Some(MapBounds::new(
                BoundingBox::try_new(s_lat, s_lng, n_lat, n_lng)?,
                ZoomLevel::new(zoom_from_db(min)?)?,
                ZoomLevel::new(zoom_from_db(max)?)?,
            )?)
        }
        _ => None,
    };
    let map_view = match (snap.map_center_lat, snap.map_center_lng) {
        (Some(lat), Some(lng)) => Some(MapView::new(Coordinate::new(lat, lng)?, bounds)?),
        _ => None,
    };

    Ok(OrganizationSettings {
        organization_id: Id::new(snap.organization_id),
        water_demand: snap.water_demand_liters.map(WaterDemand::new).transpose()?,
        just_watered_ttl: snap
            .just_watered_ttl_secs
            .map(JustWateredTtl::new)
            .transpose()?,
        sensor_offline_after: snap
            .sensor_offline_after_secs
            .map(SensorOfflineAfter::new)
            .transpose()?,
        defect_streak: snap.defect_streak.map(DefectStreak::new).transpose()?,
        map_view,
        descendants_may_override: snap.descendants_may_override,
    })
}

/// The history is display-only, so the shape mirrors what a reader wants to
/// see: a bare number for the scalar values, an object for the viewport.
fn to_json(value: SettingValue) -> serde_json::Value {
    match value {
        SettingValue::Liters(v) => serde_json::json!(v),
        SettingValue::Seconds(v) => serde_json::json!(v),
        SettingValue::Count(v) => serde_json::json!(v),
        SettingValue::Flag(v) => serde_json::json!(v),
        SettingValue::Viewport(m) => serde_json::json!({
            "center": [m.center().latitude(), m.center().longitude()],
            "bbox": m.bounds().map(|b| {
                let bbox = b.bbox();
                [bbox.sw_lat(), bbox.sw_lng(), bbox.ne_lat(), bbox.ne_lng()]
            }),
            "min_zoom": m.bounds().map(|b| b.min_zoom().level()),
            "max_zoom": m.bounds().map(|b| b.max_zoom().level()),
        }),
    }
}

#[async_trait::async_trait]
impl SettingsReader for PgSettingsRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn own(&self, org: Id<Organization>) -> Result<OrganizationSettings, RepositoryError> {
        let by_org = self.by_org().await?;
        Ok(by_org
            .get(&org)
            .copied()
            .unwrap_or_else(|| OrganizationSettings::empty(org)))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn resolution(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError> {
        self.resolve_for(org).await
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn last_changes(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<SettingKey, SettingChangeEntry>, RepositoryError> {
        // `changed_at` defaults to `now()`, which is transaction-start time, so
        // every key touched by one patch carries the same timestamp. `id` is a
        // uuid v7 and breaks the tie in insertion order; without it the planner
        // decides which row counts as the last one.
        let rows = sqlx::query!(
            r#"SELECT DISTINCT ON (setting_key)
                      setting_key AS "setting_key!", previous_value, new_value,
                      changed_at, changed_by
               FROM organization_settings_history
               WHERE organization_id = $1
               ORDER BY setting_key, changed_at DESC, id DESC"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = HashMap::new();
        for row in rows {
            // A key the current build no longer knows is skipped rather than
            // failing the whole read: the history outlives a renamed setting.
            let Ok(key) = SettingKey::from_str(&row.setting_key) else {
                continue;
            };
            out.insert(
                key,
                SettingChangeEntry {
                    organization_id: org,
                    key,
                    previous: row.previous_value,
                    next: row.new_value,
                    changed_at: row.changed_at,
                    changed_by: row.changed_by,
                },
            );
        }
        Ok(out)
    }
}

#[async_trait::async_trait]
impl SettingsWriter for PgSettingsRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn apply(
        &self,
        org: Id<Organization>,
        update: SettingsUpdate,
        actor: Option<uuid::Uuid>,
    ) -> Result<OrganizationSettings, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let current = sqlx::query_as!(
            OrganizationSettingsSnapshot,
            r#"SELECT organization_id, water_demand_liters, just_watered_ttl_secs,
                      sensor_offline_after_secs, defect_streak,
                      map_center_lat, map_center_lng,
                      map_bbox_sw_lat, map_bbox_sw_lng,
                      map_bbox_ne_lat, map_bbox_ne_lng,
                      map_min_zoom, map_max_zoom,
                      descendants_may_override
               FROM organization_settings WHERE organization_id = $1 FOR UPDATE"#,
            org.value()
        )
        .fetch_optional(&mut *tx)
        .await?
        .map(reconstitute)
        .transpose()?
        .unwrap_or_else(|| OrganizationSettings::empty(org));

        let next = update.apply_to(&current);
        let map = next.map_view;

        sqlx::query!(
            r#"INSERT INTO organization_settings (
                   organization_id, water_demand_liters, just_watered_ttl_secs,
                   sensor_offline_after_secs, defect_streak,
                   map_center_lat, map_center_lng,
                   map_bbox_sw_lat, map_bbox_sw_lng, map_bbox_ne_lat, map_bbox_ne_lng,
                   map_min_zoom, map_max_zoom,
                   descendants_may_override)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
               ON CONFLICT (organization_id) DO UPDATE SET
                   water_demand_liters = EXCLUDED.water_demand_liters,
                   just_watered_ttl_secs = EXCLUDED.just_watered_ttl_secs,
                   sensor_offline_after_secs = EXCLUDED.sensor_offline_after_secs,
                   defect_streak = EXCLUDED.defect_streak,
                   map_center_lat = EXCLUDED.map_center_lat,
                   map_center_lng = EXCLUDED.map_center_lng,
                   map_bbox_sw_lat = EXCLUDED.map_bbox_sw_lat,
                   map_bbox_sw_lng = EXCLUDED.map_bbox_sw_lng,
                   map_bbox_ne_lat = EXCLUDED.map_bbox_ne_lat,
                   map_bbox_ne_lng = EXCLUDED.map_bbox_ne_lng,
                   map_min_zoom = EXCLUDED.map_min_zoom,
                   map_max_zoom = EXCLUDED.map_max_zoom,
                   descendants_may_override = EXCLUDED.descendants_may_override"#,
            org.value(),
            next.water_demand.map(|v| v.liters()),
            next.just_watered_ttl.map(|v| v.seconds()),
            next.sensor_offline_after.map(|v| v.seconds()),
            next.defect_streak.map(|v| v.count()),
            map.map(|m| m.center().latitude()),
            map.map(|m| m.center().longitude()),
            map.and_then(|m| m.bounds()).map(|b| b.bbox().sw_lat()),
            map.and_then(|m| m.bounds()).map(|b| b.bbox().sw_lng()),
            map.and_then(|m| m.bounds()).map(|b| b.bbox().ne_lat()),
            map.and_then(|m| m.bounds()).map(|b| b.bbox().ne_lng()),
            map.and_then(|m| m.bounds())
                .map(|b| i16::from(b.min_zoom().level())),
            map.and_then(|m| m.bounds())
                .map(|b| i16::from(b.max_zoom().level())),
            next.descendants_may_override,
        )
        .execute(&mut *tx)
        .await?;

        for change in update.changes(&current) {
            sqlx::query!(
                r#"INSERT INTO organization_settings_history
                       (id, organization_id, setting_key, previous_value, new_value, changed_by)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
                uuid::Uuid::now_v7(),
                org.value(),
                change.key.as_str(),
                change.previous.map(to_json),
                change.next.map(to_json),
                actor,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(next)
    }
}

#[async_trait::async_trait]
impl SettingsResolver for PgSettingsRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn effective_for(
        &self,
        org: Id<Organization>,
    ) -> Result<EffectiveSettings, RepositoryError> {
        Ok(self.resolve_for(org).await?.effective)
    }
}
