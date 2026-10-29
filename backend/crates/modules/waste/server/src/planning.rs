use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use chrono::Utc;
use domain::{
    Id,
    authorization::ResourceKey,
    capacity::CapacityKind,
    organization::{Organization, OrganizationReader},
    planning::{PlanningError, PlanningProvider, ResolvedStop, StopRef},
};
use waste_domain::{
    fill::FillState,
    waste_bin::{WasteBin, WasteBinReader},
};

use super::{resources, service::fill_rules::FillRulesResolver};

/// Collection tours visit waste bins and carry away what they hold, so a stop's
/// demand is the share of the bin's volume that is filled.
pub struct WastePlanning {
    bins: Arc<dyn WasteBinReader>,
    organizations: Arc<dyn OrganizationReader>,
    fill_rules: Arc<FillRulesResolver>,
}

impl WastePlanning {
    pub fn new(
        bins: Arc<dyn WasteBinReader>,
        organizations: Arc<dyn OrganizationReader>,
        fill_rules: Arc<FillRulesResolver>,
    ) -> Self {
        Self {
            bins,
            organizations,
            fill_rules,
        }
    }
}

#[async_trait::async_trait]
impl PlanningProvider for WastePlanning {
    fn capacity_kind(&self) -> CapacityKind {
        CapacityKind::CargoVolume
    }

    fn plan_resource(&self) -> ResourceKey {
        resources::COLLECTION_PLAN
    }

    async fn resolve_stops(
        &self,
        stops: &[StopRef],
        org: Id<Organization>,
    ) -> Result<Vec<ResolvedStop>, PlanningError> {
        if stops.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<Id<WasteBin>> = stops.iter().map(|s| Id::new(s.value())).collect();
        let bins = self.bins.by_ids(&ids).await?;
        let hierarchy = self.organizations.hierarchy().await?;
        // A bin is judged by its own organization's rules, as on its label.
        let mut rules = HashMap::new();
        for bin in &bins {
            if let Entry::Vacant(slot) = rules.entry(bin.organization_id()) {
                slot.insert(self.fill_rules.rules_for(bin.organization_id()).await?);
            }
        }
        let now = Utc::now();
        stops
            .iter()
            .map(|stop| {
                let bin = bins
                    .iter()
                    .find(|b| b.id.value() == stop.value())
                    .ok_or(PlanningError::UnknownStop(*stop))?;
                if !hierarchy.is_descendant_or_self(bin.organization_id(), org) {
                    return Err(PlanningError::OrganizationMismatch);
                }
                let rules = rules[&bin.organization_id()];
                let volume = bin.volume.liters();
                let demand = match (
                    bin.fill_state(rules.threshold, now, rules.stale_after),
                    bin.fill(),
                ) {
                    (FillState::Empty, _) => 0.0,
                    (FillState::Full | FillState::Normal, Some(fill)) => volume * fill.fraction,
                    _ => volume,
                };
                Ok(ResolvedStop {
                    stop: *stop,
                    label: bin.number.as_str().to_owned(),
                    location: Some(bin.coordinate),
                    demand,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{EffectiveWasteSettings, WasteSettingsResolver};
    use chrono::{DateTime, Duration, Utc};
    use domain::{
        RepositoryError,
        authorization::{OrgHierarchy, Visibility},
        settings::{
            EffectiveSettings, SettingsResolver,
            values::{DefectStreak, MapView, SensorOfflineAfter},
        },
        shared::{
            coordinates::Coordinate,
            pagination::{Pagination, SearchPage},
        },
    };
    use uuid::Uuid;
    use waste_domain::{
        settings::FullThreshold,
        waste_bin::{WasteBinSearchQuery, WasteBinSnapshot, WasteBinView},
    };

    struct Bins(Vec<WasteBin>);

    #[async_trait::async_trait]
    impl WasteBinReader for Bins {
        async fn by_id(&self, _id: Id<WasteBin>) -> Result<WasteBin, RepositoryError> {
            unimplemented!()
        }
        async fn by_ids(&self, ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError> {
            Ok(self
                .0
                .iter()
                .filter(|b| ids.contains(&b.id))
                .cloned()
                .collect())
        }
        async fn view_by_id(&self, _id: Id<WasteBin>) -> Result<WasteBinView, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_ids(
            &self,
            _ids: &[Id<WasteBin>],
        ) -> Result<Vec<WasteBinView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_search(
            &self,
            _query: WasteBinSearchQuery,
            _pagination: Pagination,
            _visibility: &Visibility,
        ) -> Result<SearchPage<WasteBinView>, RepositoryError> {
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

    struct OfflineAfterOneHour;

    #[async_trait::async_trait]
    impl SettingsResolver for OfflineAfterOneHour {
        async fn effective_for(
            &self,
            _org: Id<Organization>,
        ) -> Result<EffectiveSettings, RepositoryError> {
            Ok(EffectiveSettings {
                sensor_offline_after: SensorOfflineAfter::new(3_600).unwrap(),
                defect_streak: DefectStreak::new(3).unwrap(),
                map_view: MapView::new(Coordinate::new(54.78, 9.43).unwrap(), None).unwrap(),
                enforced_by: None,
            })
        }
    }

    /// Staleness limit per organization; unknown organizations get one hour.
    struct OfflineAfterByOrg(std::collections::HashMap<Id<Organization>, i64>);

    #[async_trait::async_trait]
    impl SettingsResolver for OfflineAfterByOrg {
        async fn effective_for(
            &self,
            org: Id<Organization>,
        ) -> Result<EffectiveSettings, RepositoryError> {
            let seconds = self.0.get(&org).copied().unwrap_or(3_600);
            Ok(EffectiveSettings {
                sensor_offline_after: SensorOfflineAfter::new(seconds).unwrap(),
                defect_streak: DefectStreak::new(3).unwrap(),
                map_view: MapView::new(Coordinate::new(54.78, 9.43).unwrap(), None).unwrap(),
                enforced_by: None,
            })
        }
    }

    struct EightyPercent;

    #[async_trait::async_trait]
    impl WasteSettingsResolver for EightyPercent {
        async fn effective_for(
            &self,
            _org: Id<Organization>,
        ) -> Result<EffectiveWasteSettings, RepositoryError> {
            Ok(EffectiveWasteSettings {
                full_threshold: FullThreshold::new(80).unwrap(),
                enforced_by: None,
            })
        }
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

    fn bin(
        org: Id<Organization>,
        fill: Option<(f64, DateTime<Utc>)>,
        emptied: Option<DateTime<Utc>>,
    ) -> WasteBin {
        WasteBin::reconstitute(WasteBinSnapshot {
            id: Uuid::now_v7(),
            number: "AB-1".into(),
            latitude: 54.78,
            longitude: 9.43,
            volume_liters: 240.0,
            depth_cm: 100,
            organization_id: org.value(),
            provider: None,
            additional_informations: None,
            fill_fraction: fill.map(|f| f.0),
            fill_measured_at: fill.map(|f| f.1),
            last_emptied_at: emptied,
        })
        .unwrap()
    }

    fn planning(f: &Fixture, bins: Vec<WasteBin>) -> WastePlanning {
        WastePlanning::new(
            Arc::new(Bins(bins)),
            Arc::new(Orgs(vec![
                (f.root, None),
                (f.child, Some(f.root)),
                (f.other, None),
            ])),
            Arc::new(FillRulesResolver::new(
                Arc::new(OfflineAfterOneHour),
                Arc::new(EightyPercent),
            )),
        )
    }

    fn stop(b: &WasteBin) -> StopRef {
        StopRef::new(b.id.value())
    }

    fn hours_ago(h: i64) -> DateTime<Utc> {
        Utc::now() - Duration::hours(h)
    }

    /// The plan's organization may judge readings stale sooner than the
    /// sub-unit that owns the bin; the bin's own rules decide, as on its label.
    #[tokio::test]
    async fn a_bin_is_judged_by_its_own_organizations_staleness_limit() {
        let f = fixture();
        let b = bin(f.child, Some((0.1, hours_ago(5))), None);
        let planning = WastePlanning::new(
            Arc::new(Bins(vec![b.clone()])),
            Arc::new(Orgs(vec![(f.root, None), (f.child, Some(f.root))])),
            Arc::new(FillRulesResolver::new(
                Arc::new(OfflineAfterByOrg(std::collections::HashMap::from([(
                    f.child,
                    10 * 3_600,
                )]))),
                Arc::new(EightyPercent),
            )),
        );
        let resolved = planning.resolve_stops(&[stop(&b)], f.root).await.unwrap();
        assert!((resolved[0].demand - 24.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn demand_is_the_filled_share_of_the_volume() {
        let f = fixture();
        let b = bin(f.root, Some((0.5, hours_ago(0))), None);
        let resolved = planning(&f, vec![b.clone()])
            .resolve_stops(&[stop(&b)], f.root)
            .await
            .unwrap();
        assert_eq!(resolved[0].demand, 120.0);
        assert_eq!(resolved[0].stop, stop(&b));
        assert_eq!(resolved[0].location, Some(b.coordinate));
    }

    #[tokio::test]
    async fn a_stop_is_labelled_with_the_bin_number() {
        let f = fixture();
        let b = bin(f.root, None, None);
        let resolved = planning(&f, vec![b.clone()])
            .resolve_stops(&[stop(&b)], f.root)
            .await
            .unwrap();
        assert_eq!(resolved[0].label, "AB-1");
    }

    #[tokio::test]
    async fn a_full_bin_plans_its_measured_share() {
        let f = fixture();
        let b = bin(f.root, Some((0.9, hours_ago(0))), None);
        let resolved = planning(&f, vec![b.clone()])
            .resolve_stops(&[stop(&b)], f.root)
            .await
            .unwrap();
        assert!((resolved[0].demand - 216.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn an_unknown_fill_plans_the_whole_volume() {
        let f = fixture();
        let unmeasured = bin(f.root, None, None);
        let stale = bin(f.root, Some((0.1, hours_ago(5))), None);
        let resolved = planning(&f, vec![unmeasured.clone(), stale.clone()])
            .resolve_stops(&[stop(&unmeasured), stop(&stale)], f.root)
            .await
            .unwrap();
        assert_eq!(resolved[0].demand, 240.0);
        assert_eq!(resolved[1].demand, 240.0);
    }

    #[tokio::test]
    async fn an_emptied_bin_plans_no_demand() {
        let f = fixture();
        let b = bin(
            f.root,
            Some((0.9, Utc::now() - Duration::minutes(30))),
            Some(Utc::now() - Duration::minutes(10)),
        );
        let resolved = planning(&f, vec![b.clone()])
            .resolve_stops(&[stop(&b)], f.root)
            .await
            .unwrap();
        assert_eq!(resolved[0].demand, 0.0);
    }

    #[tokio::test]
    async fn a_bin_of_a_foreign_organization_is_refused() {
        let f = fixture();
        let foreign = bin(f.other, None, None);
        let r = planning(&f, vec![foreign.clone()])
            .resolve_stops(&[stop(&foreign)], f.root)
            .await;
        assert!(matches!(r, Err(PlanningError::OrganizationMismatch)));
    }

    #[tokio::test]
    async fn a_bin_of_a_child_organization_is_accepted() {
        let f = fixture();
        let b = bin(f.child, None, None);
        let r = planning(&f, vec![b.clone()])
            .resolve_stops(&[stop(&b)], f.root)
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn an_unknown_stop_is_refused() {
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
