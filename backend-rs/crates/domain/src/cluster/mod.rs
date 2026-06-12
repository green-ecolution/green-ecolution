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
pub mod view;

use chrono::{DateTime, Utc};

use crate::{
    Id,
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
pub use view::{ClusterBoundaryView, TreeClusterView};

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
}

#[derive(Debug, Default, Clone)]
pub struct TreeClusterSearchQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub regions: Vec<uuid::Uuid>,
    pub ids: Vec<Id<TreeCluster>>,
    pub provider: Option<ProviderId>,
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
        }
    }

    pub fn watering_status(&self) -> WateringStatus {
        self.watering_status
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

    /// Replaces the freely editable display fields. None of these have
    /// subscribers, so no event is emitted. Tree membership goes through
    /// `replace_trees` (with `ClusterTreesChanged` published from the service
    /// when the set actually changed); centroid, region, watering status and
    /// the archived flag are private and only changed through their own
    /// recalculation methods.
    pub fn replace_details(
        &mut self,
        name: ClusterName,
        address: ClusterAddress,
        description: String,
        soil_condition: Option<SoilCondition>,
        provenance: Provenance,
    ) {
        self.name = name;
        self.address = address;
        self.description = description;
        self.soil_condition = soil_condition;
        self.provenance = provenance;
    }

    pub fn replace_trees(&mut self, tree_ids: Vec<Id<Tree>>) {
        self.tree_ids = tree_ids;
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
    fn replace_trees_replaces_set() {
        let mut c = fixed_cluster();
        let t1: Id<crate::tree::Tree> = Id::new_v7();
        let t2: Id<crate::tree::Tree> = Id::new_v7();
        c.replace_trees(vec![t1, t2]);
        assert_eq!(c.tree_ids, vec![t1, t2]);
    }

    #[test]
    fn replace_details_overwrites_user_facing_fields() {
        let mut c = fixed_cluster();
        let new_provenance = Provenance::new(
            Some(crate::shared::provenance::ProviderId::new("tbz").unwrap()),
            None,
        );
        c.replace_details(
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
    }

    #[test]
    fn mark_watered_at_sets_timestamp() {
        let mut c = fixed_cluster();
        let ts = Utc::now();
        c.mark_watered_at(ts);
        assert_eq!(c.last_watered, Some(ts));
    }
}
