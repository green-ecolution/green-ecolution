use std::sync::Arc;

use domain::{
    Id,
    authorization::ResourceKey,
    capacity::CapacityKind,
    organization::{Organization, OrganizationReader},
    planning::{PlanningError, PlanningProvider, ResolvedStop, StopRef},
};
use irrigation_domain::cluster::{TreeCluster, TreeClusterReader};

use super::{resources, settings::IrrigationSettingsResolver};

/// Irrigation plans visit tree clusters; each cluster needs the configured
/// water demand per tree.
pub struct IrrigationPlanning {
    clusters: Arc<dyn TreeClusterReader>,
    organizations: Arc<dyn OrganizationReader>,
    settings: Arc<dyn IrrigationSettingsResolver>,
}

impl IrrigationPlanning {
    pub fn new(
        clusters: Arc<dyn TreeClusterReader>,
        organizations: Arc<dyn OrganizationReader>,
        settings: Arc<dyn IrrigationSettingsResolver>,
    ) -> Self {
        Self {
            clusters,
            organizations,
            settings,
        }
    }
}

#[async_trait::async_trait]
impl PlanningProvider for IrrigationPlanning {
    fn capacity_kind(&self) -> CapacityKind {
        CapacityKind::Water
    }

    fn plan_resource(&self) -> ResourceKey {
        resources::WATERING_PLAN
    }

    async fn resolve_stops(
        &self,
        stops: &[StopRef],
        org: Id<Organization>,
    ) -> Result<Vec<ResolvedStop>, PlanningError> {
        if stops.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<Id<TreeCluster>> = stops.iter().map(|s| Id::new(s.value())).collect();
        let clusters = self.clusters.by_ids(&ids).await?;
        let hierarchy = self.organizations.hierarchy().await?;
        let liters_per_tree = self
            .settings
            .effective_for(org)
            .await?
            .water_demand
            .liters();
        stops
            .iter()
            .map(|stop| {
                let cluster = clusters
                    .iter()
                    .find(|c| c.id.value() == stop.value())
                    .ok_or(PlanningError::UnknownStop(*stop))?;
                if !hierarchy.is_descendant_or_self(cluster.organization_id(), org) {
                    return Err(PlanningError::OrganizationMismatch);
                }
                Ok(ResolvedStop {
                    stop: *stop,
                    location: cluster.coordinates(),
                    demand: cluster.tree_ids.len() as f64 * liters_per_tree,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::EffectiveIrrigationSettings;
    use chrono::{DateTime, Utc};
    use domain::{
        RepositoryError,
        authorization::{OrgHierarchy, Visibility},
        sensor::data::{SoilMoistureBucket, SoilMoistureDepthSeries},
        shared::{
            coordinates::Coordinate,
            pagination::{Pagination, SearchPage},
        },
    };
    use irrigation_domain::{
        cluster::{
            ClusterBoundaryView, ClusterMarker, ClusterStatistics, ClusterWateringEvent,
            TreeClusterSearchQuery, TreeClusterView, snapshot::TreeClusterSnapshot,
        },
        settings::{JustWateredTtl, WaterDemand},
        watering_status::WateringStatus,
    };
    use uuid::Uuid;

    struct Clusters(Vec<TreeCluster>);

    #[async_trait::async_trait]
    impl TreeClusterReader for Clusters {
        async fn by_id(&self, _id: Id<TreeCluster>) -> Result<TreeCluster, RepositoryError> {
            unimplemented!()
        }
        async fn by_ids(
            &self,
            ids: &[Id<TreeCluster>],
        ) -> Result<Vec<TreeCluster>, RepositoryError> {
            Ok(self
                .0
                .iter()
                .filter(|c| ids.contains(&c.id))
                .cloned()
                .collect())
        }
        async fn view_by_id(
            &self,
            _id: Id<TreeCluster>,
        ) -> Result<TreeClusterView, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_ids(
            &self,
            _ids: &[Id<TreeCluster>],
        ) -> Result<Vec<TreeClusterView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_search(
            &self,
            _query: TreeClusterSearchQuery,
            _pagination: Pagination,
        ) -> Result<SearchPage<TreeClusterView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_markers(
            &self,
            _visible: Visibility,
        ) -> Result<Vec<ClusterMarker>, RepositoryError> {
            unimplemented!()
        }
        async fn boundaries(
            &self,
            _visible: Visibility,
        ) -> Result<Vec<ClusterBoundaryView>, RepositoryError> {
            unimplemented!()
        }
        async fn center_point(
            &self,
            _id: Id<TreeCluster>,
        ) -> Result<Option<Coordinate>, RepositoryError> {
            unimplemented!()
        }
        async fn statistics(
            &self,
            _visible: Visibility,
        ) -> Result<ClusterStatistics, RepositoryError> {
            unimplemented!()
        }
        async fn soil_moisture_series(
            &self,
            _id: Id<TreeCluster>,
            _from: DateTime<Utc>,
            _to: DateTime<Utc>,
            _bucket: SoilMoistureBucket,
        ) -> Result<Vec<SoilMoistureDepthSeries>, RepositoryError> {
            unimplemented!()
        }
        async fn just_watered_before(
            &self,
            _org: Id<Organization>,
            _cutoff: DateTime<Utc>,
        ) -> Result<Vec<TreeCluster>, RepositoryError> {
            unimplemented!()
        }
        async fn watering_events(
            &self,
            _id: Id<TreeCluster>,
        ) -> Result<Vec<ClusterWateringEvent>, RepositoryError> {
            unimplemented!()
        }
    }

    struct Orgs(Vec<(Id<Organization>, Option<Id<Organization>>)>);

    #[async_trait::async_trait]
    impl OrganizationReader for Orgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, _id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs(self.0.clone()))
        }
        async fn member_counts(
            &self,
        ) -> Result<std::collections::HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(std::collections::HashMap::new())
        }
        async fn resource_counts(
            &self,
        ) -> Result<
            std::collections::HashMap<
                Id<Organization>,
                domain::organization::OrganizationResourceCounts,
            >,
            RepositoryError,
        > {
            Ok(std::collections::HashMap::new())
        }
    }

    struct Liters(f64);

    #[async_trait::async_trait]
    impl IrrigationSettingsResolver for Liters {
        async fn effective_for(
            &self,
            _org: Id<Organization>,
        ) -> Result<EffectiveIrrigationSettings, RepositoryError> {
            Ok(EffectiveIrrigationSettings {
                water_demand: WaterDemand::new(self.0).unwrap(),
                just_watered_ttl: JustWateredTtl::new(86_400).unwrap(),
                enforced_by: None,
            })
        }
    }

    fn cluster(org: Id<Organization>, trees: usize) -> TreeCluster {
        TreeCluster::reconstitute(TreeClusterSnapshot {
            id: Uuid::now_v7(),
            name: "Cluster".into(),
            address: "Stadtpark 1".into(),
            description: String::new(),
            watering_status: WateringStatus::Unknown,
            last_watered: None,
            moisture_level: 0.0,
            region_id: None,
            archived: false,
            latitude: Some(54.78),
            longitude: Some(9.43),
            soil_condition: None,
            tree_ids: (0..trees).map(|_| Uuid::now_v7()).collect(),
            provider: None,
            additional_info: None,
            organization_id: org.value(),
        })
    }

    struct Fixture {
        root: Id<Organization>,
        child: Id<Organization>,
        other: Id<Organization>,
    }

    fn fixture() -> Fixture {
        Fixture {
            root: Id::new_v7(),
            child: Id::new_v7(),
            other: Id::new_v7(),
        }
    }

    fn planning(f: &Fixture, clusters: Vec<TreeCluster>) -> IrrigationPlanning {
        IrrigationPlanning::new(
            Arc::new(Clusters(clusters)),
            Arc::new(Orgs(vec![
                (f.root, None),
                (f.child, Some(f.root)),
                (f.other, None),
            ])),
            Arc::new(Liters(80.0)),
        )
    }

    fn stop(c: &TreeCluster) -> StopRef {
        StopRef::new(c.id.value())
    }

    #[tokio::test]
    async fn demand_is_trees_times_liters_per_tree() {
        let f = fixture();
        let (a, b) = (cluster(f.root, 3), cluster(f.child, 0));
        let resolved = planning(&f, vec![a.clone(), b.clone()])
            .resolve_stops(&[stop(&b), stop(&a)], f.root)
            .await
            .unwrap();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].stop, stop(&b));
        assert_eq!(resolved[0].demand, 0.0);
        assert_eq!(resolved[1].stop, stop(&a));
        assert_eq!(resolved[1].demand, 240.0);
        assert_eq!(resolved[1].location, a.coordinates());
    }

    #[tokio::test]
    async fn a_cluster_outside_the_plan_subtree_is_a_mismatch() {
        let f = fixture();
        let foreign = cluster(f.other, 1);
        let r = planning(&f, vec![foreign.clone()])
            .resolve_stops(&[stop(&foreign)], f.root)
            .await;
        assert!(matches!(r, Err(PlanningError::OrganizationMismatch)));
    }

    #[tokio::test]
    async fn an_unknown_cluster_is_an_unknown_stop() {
        let f = fixture();
        let ghost = StopRef::new(Uuid::now_v7());
        let r = planning(&f, vec![]).resolve_stops(&[ghost], f.root).await;
        assert!(matches!(r, Err(PlanningError::UnknownStop(s)) if s == ghost));
    }

    #[tokio::test]
    async fn no_stops_resolve_to_nothing() {
        let f = fixture();
        let resolved = planning(&f, vec![]).resolve_stops(&[], f.root).await;
        assert!(resolved.unwrap().is_empty());
    }
}
