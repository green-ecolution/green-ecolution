//! TreeCluster aggregate — a named group of trees that are watered together.
//!
//! Three fields are private because they are derived rather than user-supplied:
//!
//! - `watering_status` — majority vote across member trees, updated by
//!   `recalculate_watering_status`.
//! - `coordinates` — centroid of member tree coordinates, updated by
//!   `recalculate_centroid`. Cleared when the cluster has no trees.
//! - `region_id` — set by `assign_region` after a spatial lookup maps the
//!   centroid to a region polygon.
//!
//! [`TreeClusterView`] adds audit fields for HTTP responses.

pub mod error;
pub mod marker;
pub mod repository;
pub mod snapshot;
pub mod soil_condition;
pub mod soil_moisture;
pub mod view;

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use crate::{
    Id,
    authorization::Visibility,
    events::DomainEvent,
    organization::Organization,
    region::Region,
    shared::{
        coordinates::Coordinate,
        provenance::{Provenance, ProviderId},
        watering_status::WateringStatus,
    },
    tree::Tree,
};

pub use error::ClusterError;
pub use marker::ClusterMarker;
pub use repository::{TreeClusterReader, TreeClusterWriter};
#[doc(hidden)]
pub use snapshot::TreeClusterSnapshot;
pub use soil_condition::SoilCondition;
pub use soil_moisture::{
    ClusterWateringEvent, SoilMoistureBucket, SoilMoistureConditionPoint, SoilMoistureDepthSeries,
    SoilMoistureOverview, SoilMoisturePoint, condition_series,
};
pub use view::{ClusterBoundaryView, TreeClusterView};

#[derive(Debug, Clone, Default)]
pub struct ClusterStatistics {
    pub total: i64,
    pub trees: i64,
    pub bad: i64,
    pub moderate: i64,
    pub good: i64,
    pub just_watered: i64,
    pub unknown: i64,
}

crate::newtype_nonempty! {
    /// Human-readable cluster name, 1–255 characters after trimming.
    ClusterName, "cluster.name", 1, 255
}

crate::newtype_nonempty! {
    /// Street address or location description for a cluster, 1–512 characters.
    ClusterAddress, "cluster.address", 1, 512
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCluster {
    pub id: Id<TreeCluster>,
    pub name: ClusterName,
    pub address: ClusterAddress,
    pub description: String,
    pub moisture_level: f64,
    pub last_watered: Option<DateTime<Utc>>,
    pub soil_condition: Option<SoilCondition>,
    pub tree_ids: Vec<Id<Tree>>,

    watering_status: WateringStatus,
    coordinates: Option<Coordinate>,
    region_id: Option<Id<Region>>,
    archived: bool,
    provenance: Provenance,
    organization_id: Id<Organization>,
    shared_with: BTreeSet<Id<Organization>>,
}

/// Input for creating a new [`TreeCluster`].
#[derive(Debug, Clone)]
pub struct TreeClusterDraft {
    pub name: ClusterName,
    pub address: ClusterAddress,
    pub description: String,
    pub moisture_level: f64,
    pub soil_condition: Option<SoilCondition>,
    pub tree_ids: Vec<Id<Tree>>,
    pub provenance: Provenance,
    pub organization_id: Id<Organization>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClusterSort {
    #[default]
    Name,
    Moisture,
    Trees,
}

impl ClusterSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClusterSort::Name => "name",
            ClusterSort::Moisture => "moisture",
            ClusterSort::Trees => "trees",
        }
    }
}

impl std::str::FromStr for ClusterSort {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(ClusterSort::Name),
            "moisture" => Ok(ClusterSort::Moisture),
            "trees" => Ok(ClusterSort::Trees),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

impl std::str::FromStr for SortOrder {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TreeClusterSearchQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub regions: Vec<uuid::Uuid>,
    pub ids: Vec<Id<TreeCluster>>,
    pub provider: Option<ProviderId>,
    pub query: Option<String>,
    pub soil_conditions: Vec<SoilCondition>,
    pub sort: ClusterSort,
    pub order: SortOrder,
    /// Which organizations may see the result. Defaults to `Unrestricted` so
    /// internal callers (event handlers) stay unfiltered; HTTP handlers must
    /// set this from `AuthorizationService::visible_orgs_for`.
    pub visible: Visibility,
}

/// Replacement input for [`TreeCluster`] updates.
#[derive(Debug, Clone)]
pub struct TreeClusterUpdate {
    pub name: ClusterName,
    pub address: ClusterAddress,
    pub description: String,
    pub soil_condition: Option<SoilCondition>,
    pub tree_ids: Vec<Id<Tree>>,
    pub provenance: Provenance,
}

impl TreeCluster {
    #[doc(hidden)]
    pub fn reconstitute(snap: TreeClusterSnapshot) -> Self {
        let coordinates = match (snap.latitude, snap.longitude) {
            (Some(lat), Some(lng)) => {
                Some(Coordinate::new(lat, lng).expect("DB coordinate values must be valid"))
            }
            _ => None,
        };
        Self {
            id: Id::new(snap.id),
            name: ClusterName::reconstitute(snap.name),
            address: ClusterAddress::reconstitute(snap.address),
            description: snap.description,
            moisture_level: snap.moisture_level,
            last_watered: snap.last_watered,
            soil_condition: snap.soil_condition,
            tree_ids: snap.tree_ids.into_iter().map(Id::new).collect(),
            watering_status: snap.watering_status,
            coordinates,
            region_id: snap.region_id.map(Id::new),
            archived: snap.archived,
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
            organization_id: Id::new(snap.organization_id),
            shared_with: snap.shared_with.into_iter().map(Id::new).collect(),
        }
    }

    pub fn watering_status(&self) -> WateringStatus {
        self.watering_status
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn coordinates(&self) -> Option<Coordinate> {
        self.coordinates
    }

    pub fn region_id(&self) -> Option<Id<Region>> {
        self.region_id
    }

    pub fn archived(&self) -> bool {
        self.archived
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn shared_with(&self) -> &BTreeSet<Id<Organization>> {
        &self.shared_with
    }

    /// Grants access to `org` as if the cluster were its own. No-op (and no
    /// event) if the share already exists.
    pub fn share_with(&mut self, org: Id<Organization>) -> Vec<DomainEvent> {
        if !self.shared_with.insert(org) {
            return vec![];
        }
        vec![DomainEvent::ClusterSharedWithOrganization {
            cluster_id: self.id,
            organization_id: org,
        }]
    }

    /// Revokes a previously granted share. No-op (and no event) if `org`
    /// was never shared with.
    pub fn revoke_share(&mut self, org: Id<Organization>) -> Vec<DomainEvent> {
        if !self.shared_with.remove(&org) {
            return vec![];
        }
        vec![DomainEvent::ClusterShareRevoked {
            cluster_id: self.id,
            organization_id: org,
        }]
    }

    /// Replaces the freely editable display fields. Emits
    /// `ClusterSoilConditionChanged` when `soil_condition` actually changes;
    /// the other fields have no subscribers and change silently. Tree
    /// membership goes through `replace_trees` (which emits
    /// `ClusterTreesChanged` when the set actually changed); centroid,
    /// region, watering status and the archived flag are private and only
    /// changed through their own recalculation methods.
    pub fn replace_details(
        &mut self,
        name: ClusterName,
        address: ClusterAddress,
        description: String,
        soil_condition: Option<SoilCondition>,
        provenance: Provenance,
    ) -> Vec<DomainEvent> {
        let mut events = Vec::new();
        if self.soil_condition != soil_condition {
            events.push(DomainEvent::ClusterSoilConditionChanged {
                cluster_id: self.id,
            });
        }
        self.name = name;
        self.address = address;
        self.description = description;
        self.soil_condition = soil_condition;
        self.provenance = provenance;
        events
    }

    /// Replaces the tree membership. Emits `ClusterTreesChanged` only when the
    /// membership actually changes as a *set* — the same trees in a different
    /// request order are a no-op.
    pub fn replace_trees(&mut self, tree_ids: Vec<Id<Tree>>) -> Vec<DomainEvent> {
        let old: std::collections::HashSet<Id<Tree>> = self.tree_ids.iter().copied().collect();
        let new: std::collections::HashSet<Id<Tree>> = tree_ids.iter().copied().collect();
        self.tree_ids = tree_ids;
        if old == new {
            return Vec::new();
        }
        vec![DomainEvent::ClusterTreesChanged {
            cluster_id: self.id,
        }]
    }

    /// Recalculates the geographic centroid from the given coordinates.
    ///
    /// Passing an empty slice clears `coordinates` (the cluster has no spatial
    /// position when it has no trees).
    pub fn recalculate_centroid(&mut self, coords: &[Coordinate]) {
        if coords.is_empty() {
            self.coordinates = None;
            return;
        }
        let sum_lat: f64 = coords.iter().map(|c| c.latitude()).sum();
        let sum_lng: f64 = coords.iter().map(|c| c.longitude()).sum();
        let n = coords.len() as f64;
        let lat = sum_lat / n;
        let lng = sum_lng / n;
        self.coordinates = Coordinate::new(lat, lng).ok();
    }

    pub fn assign_region(&mut self, region_id: Option<Id<Region>>) {
        self.region_id = region_id;
    }

    /// Sets `watering_status` to the most frequent value in `statuses`.
    ///
    /// Empty input maps to [`WateringStatus::Unknown`]. Ties are broken by
    /// severity (worst wins): `Bad > Moderate > JustWatered > Good > Unknown`.
    /// This makes the result deterministic and biases the cluster status
    /// toward the more alarming reading when sensors disagree.
    pub fn recalculate_watering_status(&mut self, statuses: &[WateringStatus]) {
        if statuses.is_empty() {
            self.watering_status = WateringStatus::Unknown;
            return;
        }
        let mut counts = std::collections::HashMap::<WateringStatus, usize>::new();
        for s in statuses {
            *counts.entry(*s).or_insert(0) += 1;
        }
        self.watering_status = counts
            .into_iter()
            .max_by_key(|(s, c)| (*c, severity(*s)))
            .map(|(s, _)| s)
            .unwrap_or(WateringStatus::Unknown);
    }

    pub fn mark_watered_at(&mut self, ts: DateTime<Utc>) {
        self.last_watered = Some(ts);
    }

    /// Reassigns the cluster to `target`'s organization. No-op (and no event)
    /// if it already belongs there. Callers must cascade the transfer to
    /// member trees (and their attached sensors) and revoke shares that no
    /// longer point below the new owner (see `ClusterService::transfer`).
    pub fn transfer_to(&mut self, target: Id<Organization>) -> Vec<DomainEvent> {
        if self.organization_id == target {
            return vec![];
        }
        let from = self.organization_id;
        self.organization_id = target;
        vec![DomainEvent::ClusterResponsibilityTransferred {
            cluster_id: self.id,
            from,
            to: target,
        }]
    }
}

fn severity(s: WateringStatus) -> u8 {
    match s {
        WateringStatus::Bad => 4,
        WateringStatus::Moderate => 3,
        WateringStatus::JustWatered => 2,
        WateringStatus::Good => 1,
        WateringStatus::Unknown => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_cluster() -> TreeCluster {
        TreeCluster {
            id: Id::new_v7(),
            name: ClusterName::new("Cluster Stadtpark Nord").unwrap(),
            address: ClusterAddress::new("Stadtpark 1, 24937 Flensburg").unwrap(),
            description: "Baumgruppe".to_string(),
            moisture_level: 0.5,
            last_watered: None,
            soil_condition: Some(SoilCondition::Lu),
            tree_ids: vec![],
            watering_status: WateringStatus::Unknown,
            coordinates: None,
            region_id: None,
            archived: false,
            provenance: Provenance::default(),
            organization_id: Id::new_v7(),
            shared_with: BTreeSet::new(),
        }
    }

    #[test]
    fn cluster_name_rejects_empty() {
        assert_err!(ClusterName::new(""));
    }

    #[test]
    fn cluster_name_accepts_valid() {
        assert_ok!(ClusterName::new("Cluster"));
    }

    #[test]
    fn cluster_address_rejects_empty() {
        assert_err!(ClusterAddress::new(""));
    }

    #[test]
    fn recalculate_centroid_with_no_coords_clears() {
        let mut c = fixed_cluster();
        c.coordinates = Coordinate::new(54.0, 9.0).ok();
        c.recalculate_centroid(&[]);
        assert!(c.coordinates().is_none());
    }

    #[test]
    fn recalculate_centroid_averages_coords() {
        let mut c = fixed_cluster();
        let coords = vec![
            Coordinate::new(54.0, 9.0).unwrap(),
            Coordinate::new(56.0, 11.0).unwrap(),
        ];
        c.recalculate_centroid(&coords);
        let result = c.coordinates().unwrap();
        assert!((result.latitude() - 55.0).abs() < 1e-9);
        assert!((result.longitude() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn recalculate_watering_status_picks_majority() {
        let mut c = fixed_cluster();
        let statuses = vec![
            WateringStatus::Good,
            WateringStatus::Good,
            WateringStatus::Bad,
        ];
        c.recalculate_watering_status(&statuses);
        assert_eq!(c.watering_status(), WateringStatus::Good);
    }

    #[test]
    fn recalculate_watering_status_with_empty_returns_unknown() {
        let mut c = fixed_cluster();
        c.watering_status = WateringStatus::Good;
        c.recalculate_watering_status(&[]);
        assert_eq!(c.watering_status(), WateringStatus::Unknown);
    }

    #[test]
    fn recalculate_watering_status_breaks_ties_by_worst_severity() {
        let mut c = fixed_cluster();
        c.recalculate_watering_status(&[WateringStatus::Good, WateringStatus::Bad]);
        assert_eq!(
            c.watering_status(),
            WateringStatus::Bad,
            "1×good vs 1×bad must resolve to Bad (worst wins on tie)"
        );

        c.recalculate_watering_status(&[
            WateringStatus::Good,
            WateringStatus::Moderate,
            WateringStatus::JustWatered,
        ]);
        assert_eq!(
            c.watering_status(),
            WateringStatus::Moderate,
            "1×good vs 1×moderate vs 1×just_watered must resolve to Moderate"
        );
    }

    #[test]
    fn assign_region_sets_id() {
        let mut c = fixed_cluster();
        let region: Id<crate::region::Region> = Id::new_v7();
        c.assign_region(Some(region));
        assert_eq!(c.region_id(), Some(region));
    }

    #[test]
    fn replace_trees_replaces_set_and_emits_event() {
        let mut c = fixed_cluster();
        let t1: Id<crate::tree::Tree> = Id::new_v7();
        let t2: Id<crate::tree::Tree> = Id::new_v7();
        let events = c.replace_trees(vec![t1, t2]);
        assert_eq!(c.tree_ids, vec![t1, t2]);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::ClusterTreesChanged { cluster_id } if cluster_id == c.id
        ));
    }

    #[test]
    fn replace_trees_same_set_in_different_order_emits_no_event() {
        let mut c = fixed_cluster();
        let t1: Id<crate::tree::Tree> = Id::new_v7();
        let t2: Id<crate::tree::Tree> = Id::new_v7();
        let _ = c.replace_trees(vec![t1, t2]);
        let events = c.replace_trees(vec![t2, t1]);
        assert!(
            events.is_empty(),
            "same membership in request order must not emit an event"
        );
    }

    #[test]
    fn replace_trees_removal_emits_event() {
        let mut c = fixed_cluster();
        let t1: Id<crate::tree::Tree> = Id::new_v7();
        let t2: Id<crate::tree::Tree> = Id::new_v7();
        let _ = c.replace_trees(vec![t1, t2]);
        let events = c.replace_trees(vec![t1]);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn replace_details_overwrites_user_facing_fields() {
        let mut c = fixed_cluster();
        let new_provenance = Provenance::new(
            Some(crate::shared::provenance::ProviderId::new("tbz").unwrap()),
            None,
        );
        // Lu → Ss: soil changed, expect one event.
        let events = c.replace_details(
            ClusterName::new("Neuer Name").unwrap(),
            ClusterAddress::new("Neue Adresse 5").unwrap(),
            "Neue Beschreibung".to_string(),
            Some(SoilCondition::Ss),
            new_provenance.clone(),
        );
        assert_eq!(c.name.as_str(), "Neuer Name");
        assert_eq!(c.address.as_str(), "Neue Adresse 5");
        assert_eq!(c.description, "Neue Beschreibung");
        assert_eq!(c.soil_condition, Some(SoilCondition::Ss));
        assert_eq!(c.provenance(), &new_provenance);
        assert_eq!(
            events.len(),
            1,
            "soil changed: expected ClusterSoilConditionChanged"
        );
        assert!(matches!(
            events[0],
            crate::events::DomainEvent::ClusterSoilConditionChanged { cluster_id }
                if cluster_id == c.id
        ));
    }

    #[test]
    fn replace_details_emits_no_event_when_soil_unchanged() {
        let mut c = fixed_cluster(); // soil_condition = Some(Lu)
        let provenance = c.provenance().clone();
        let events = c.replace_details(
            c.name.clone(),
            c.address.clone(),
            c.description.clone(),
            Some(SoilCondition::Lu),
            provenance,
        );
        assert!(events.is_empty(), "same soil value must not emit an event");
    }

    #[test]
    fn mark_watered_at_sets_timestamp() {
        let mut c = fixed_cluster();
        let ts = Utc::now();
        c.mark_watered_at(ts);
        assert_eq!(c.last_watered, Some(ts));
    }

    #[test]
    fn share_with_emits_event_and_is_idempotent() {
        let mut c = fixed_cluster();
        let org: Id<Organization> = Id::new_v7();
        let events = c.share_with(org);
        assert!(c.shared_with().contains(&org));
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::ClusterSharedWithOrganization { .. }
        ));
        assert!(c.share_with(org).is_empty(), "second share is a no-op");
    }

    #[test]
    fn revoke_share_emits_event_only_when_present() {
        let mut c = fixed_cluster();
        let org: Id<Organization> = Id::new_v7();
        assert!(c.revoke_share(org).is_empty());
        let _ = c.share_with(org);
        let events = c.revoke_share(org);
        assert!(!c.shared_with().contains(&org));
        assert!(matches!(events[0], DomainEvent::ClusterShareRevoked { .. }));
    }

    #[test]
    fn transfer_to_same_org_is_noop() {
        let mut c = fixed_cluster();
        let same = c.organization_id();
        let events = c.transfer_to(same);
        assert_eq!(c.organization_id(), same);
        assert!(events.is_empty());
    }

    #[test]
    fn transfer_to_new_org_emits_event_and_updates_field() {
        let mut c = fixed_cluster();
        let from = c.organization_id();
        let target: Id<Organization> = Id::new_v7();
        let events = c.transfer_to(target);
        assert_eq!(c.organization_id(), target);
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::ClusterResponsibilityTransferred {
                cluster_id,
                from: ev_from,
                to,
            } => {
                assert_eq!(*cluster_id, c.id);
                assert_eq!(*ev_from, from);
                assert_eq!(*to, target);
            }
            other => panic!("expected ClusterResponsibilityTransferred, got {other:?}"),
        }
    }
}
