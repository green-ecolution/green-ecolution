use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sqlx::PgPool;

use domain::{
    Id, RepositoryError,
    organization::{Organization, OrganizationReader},
    settings::{
        EffectiveSettings, InstanceDefaults, OrganizationSettings, OrganizationSettingsSnapshot,
        Resolution, SettingChangeEntry, SettingKey, SettingsReader, SettingsResolver,
        values::{DefectStreak, JustWateredTtl, MapView, SensorOfflineAfter, WaterDemand},
    },
    shared::{coordinates::Coordinate, geo::BoundingBox},
};

type SettingsByOrg = HashMap<Id<Organization>, OrganizationSettings>;

pub struct PgSettingsRepository {
    pool: PgPool,
    defaults: InstanceDefaults,
    /// The tree is read per resolution rather than cached alongside the rows:
    /// organizations are created and deleted without going through this
    /// adapter, so a cached tree would silently miss them until a restart.
    org_reader: Arc<dyn OrganizationReader>,
    /// Holds the stored rows only. Their sole writer is this adapter, which is
    /// what makes caching them safe.
    cache: RwLock<Option<Arc<SettingsByOrg>>>,
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
            cache: RwLock::new(None),
        }
    }

    async fn by_org(&self) -> Result<Arc<SettingsByOrg>, RepositoryError> {
        if let Ok(guard) = self.cache.read()
            && let Some(cached) = guard.as_ref()
        {
            return Ok(cached.clone());
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

        let mut by_org = SettingsByOrg::new();
        for row in rows {
            let settings = reconstitute(row)?;
            by_org.insert(settings.organization_id, settings);
        }

        let by_org = Arc::new(by_org);
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(by_org.clone());
        }
        Ok(by_org)
    }

    async fn resolve_for(&self, org: Id<Organization>) -> Result<Resolution, RepositoryError> {
        let by_org = self.by_org().await?;
        let hierarchy = self.org_reader.hierarchy().await?;
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

    /// Stub until the history table lands with the writer that fills it.
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
