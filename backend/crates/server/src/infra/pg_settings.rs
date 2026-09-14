use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    authorization::OrgHierarchy,
    organization::Organization,
    settings::{
        EffectiveSettings, InstanceDefaults, OrganizationSettings, OrganizationSettingsSnapshot,
        Resolution, SettingChangeEntry, SettingKey, SettingsReader, SettingsResolver,
        values::{DefectStreak, JustWateredTtl, MapView, SensorOfflineAfter, WaterDemand},
    },
    shared::{coordinates::Coordinate, geo::BoundingBox},
};

/// Everything the resolution needs, loaded together and cached as one unit.
#[derive(Clone, Default)]
struct Loaded {
    hierarchy: OrgHierarchy,
    by_org: HashMap<Id<Organization>, OrganizationSettings>,
}

pub struct PgSettingsRepository {
    pool: PgPool,
    defaults: InstanceDefaults,
    /// Cleared as a whole on every write: a change at one organization moves
    /// the effective values of its entire subtree, and flipping the lock even
    /// more so. At a few dozen organizations a clever partial invalidation
    /// would be a source of bugs without a payoff.
    cache: RwLock<Option<Arc<Loaded>>>,
}

impl PgSettingsRepository {
    pub fn new(pool: PgPool, defaults: InstanceDefaults) -> Self {
        Self {
            pool,
            defaults,
            cache: RwLock::new(None),
        }
    }

    async fn loaded(&self) -> Result<Arc<Loaded>, RepositoryError> {
        if let Ok(guard) = self.cache.read()
            && let Some(loaded) = guard.as_ref()
        {
            return Ok(loaded.clone());
        }

        let rows = sqlx::query_as!(
            OrganizationSettingsSnapshot,
            r#"SELECT organization_id, water_demand_liters, just_watered_ttl_secs,
                      sensor_offline_after_secs, defect_streak,
                      map_center_lat, map_center_lng,
                      map_bbox_sw_lat, map_bbox_sw_lng,
                      map_bbox_ne_lat, map_bbox_ne_lng,
                      descendants_may_override
               FROM organization_settings"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_org = HashMap::new();
        for row in rows {
            let settings = reconstitute(row)?;
            by_org.insert(settings.organization_id, settings);
        }

        let parents = sqlx::query!(r#"SELECT id, parent_id FROM organizations"#)
            .fetch_all(&self.pool)
            .await?;
        let hierarchy = OrgHierarchy::from_pairs(parents.into_iter().map(|r| {
            (
                Id::<Organization>::new(r.id),
                r.parent_id.map(Id::<Organization>::new),
            )
        }));

        let loaded = Arc::new(Loaded { hierarchy, by_org });
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(loaded.clone());
        }
        Ok(loaded)
    }

    async fn resolve_for(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError> {
        let loaded = self.loaded().await?;
        let chain: Vec<OrganizationSettings> = loaded
            .hierarchy
            .ancestors_from_root(org)
            .into_iter()
            .map(|id| {
                loaded
                    .by_org
                    .get(&id)
                    .copied()
                    .unwrap_or_else(|| OrganizationSettings::empty(id))
            })
            .collect();
        Ok(domain::settings::resolve(&self.defaults, &chain, org))
    }
}

/// Rehydration may fail only where the stored row breaks a domain rule the
/// schema cannot express; that is a data-integrity fault, not a not-found.
fn reconstitute(
    snap: OrganizationSettingsSnapshot,
) -> Result<OrganizationSettings, RepositoryError> {
    let map_view = match (
        snap.map_center_lat,
        snap.map_center_lng,
        snap.map_bbox_sw_lat,
        snap.map_bbox_sw_lng,
        snap.map_bbox_ne_lat,
        snap.map_bbox_ne_lng,
    ) {
        (Some(lat), Some(lng), Some(s_lat), Some(s_lng), Some(n_lat), Some(n_lng)) => {
            Some(MapView::new(
                Coordinate::new(lat, lng)?,
                BoundingBox::try_new(s_lat, s_lng, n_lat, n_lng)?,
            )?)
        }
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

#[async_trait::async_trait]
impl SettingsReader for PgSettingsRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn own(&self, org: Id<Organization>) -> Result<OrganizationSettings, RepositoryError> {
        let loaded = self.loaded().await?;
        Ok(loaded
            .by_org
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
        _org: Id<Organization>,
    ) -> Result<HashMap<SettingKey, SettingChangeEntry>, RepositoryError> {
        Ok(HashMap::new())
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
