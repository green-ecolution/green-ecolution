//! Tree aggregate — individual street / park trees managed by the platform.
//!
//! The aggregate ([`Tree`]) keeps `cluster_id`, `sensor_id`, and
//! `watering_status` private because they must only change through the
//! dedicated methods `move_to_cluster`, `attach_sensor` / `detach_sensor`, and
//! `record_watering_status`. Direct field mutation would silently bypass
//! invariants (notably: detaching a sensor resets `watering_status` to
//! `Unknown`).
//!
//! [`TreeView`] adds audit fields for HTTP responses. [`TreeViewWithDistance`]
//! is returned by proximity searches.

pub mod error;
pub mod marker;
pub mod planting_year;
pub mod repository;
pub mod snapshot;
pub mod view;
mod volumetric_calibration;
mod watermark_calibration;

use chrono::{DateTime, Datelike, Utc};

use crate::{
    Id,
    cluster::TreeCluster,
    events::DomainEvent,
    sensor::{SensorId, data::Watermark},
    shared::{
        coordinates::Coordinate,
        geo::BoundingBox,
        provenance::{Provenance, ProviderId},
        watering_status::WateringStatus,
    },
};

pub use error::TreeError;
pub use marker::TreeMarker;
pub use planting_year::PlantingYear;
pub use repository::{TreeReader, TreeWriter};
#[doc(hidden)]
pub use snapshot::TreeSnapshot;
pub use view::{TreeView, TreeViewWithDistance};

crate::newtype_nonempty! {
    /// Botanical or common species name, 1–255 characters after trimming.
    Species, "tree.species", 1, 255
}

crate::newtype_nonempty! {
    /// Municipality-assigned tree identifier (e.g. `"FL-001"`), 1–64 characters.
    TreeNumber, "tree.number", 1, 64
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    pub id: Id<Tree>,
    pub planting_year: PlantingYear,
    pub species: Species,
    pub tree_number: TreeNumber,
    pub coordinate: Coordinate,
    pub description: Option<String>,
    pub last_watered: Option<DateTime<Utc>>,

    cluster_id: Option<Id<TreeCluster>>,
    sensor_id: Option<SensorId>,
    watering_status: WateringStatus,
    provenance: Provenance,
}

/// Input for creating a new [`Tree`].
#[derive(Debug, Clone)]
pub struct TreeDraft {
    pub planting_year: PlantingYear,
    pub species: Species,
    pub tree_number: TreeNumber,
    pub coordinate: Coordinate,
    pub description: Option<String>,
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<SensorId>,
    pub provenance: Provenance,
}

/// Filter inputs for tree list queries.
#[derive(Debug, Default, Clone)]
pub struct TreeSearchQuery {
    pub watering_statuses: Vec<WateringStatus>,
    pub has_cluster: Option<bool>,
    pub planting_years: Vec<PlantingYear>,
    pub ids: Vec<Id<Tree>>,
    pub cluster_id: Option<Id<TreeCluster>>,
    pub sensor_id: Option<SensorId>,
    pub provider: Option<ProviderId>,
    pub bbox: Option<BoundingBox>,
}

impl Tree {
    #[doc(hidden)]
    pub fn reconstitute(snap: TreeSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            planting_year: PlantingYear::reconstitute(snap.planting_year as u32),
            species: Species::reconstitute(snap.species),
            tree_number: TreeNumber::reconstitute(snap.tree_number),
            coordinate: Coordinate::new(snap.latitude, snap.longitude)
                .expect("DB coordinate values must be valid"),
            description: snap.description,
            last_watered: snap.last_watered,
            cluster_id: snap.cluster_id.map(Id::new),
            sensor_id: snap.sensor_id.map(SensorId::reconstitute),
            watering_status: snap.watering_status,
            provenance: Provenance::reconstitute(snap.provider, snap.additional_info),
        }
    }

    pub fn cluster_id(&self) -> Option<Id<TreeCluster>> {
        self.cluster_id
    }

    pub fn sensor_id(&self) -> Option<&SensorId> {
        self.sensor_id.as_ref()
    }

    /// True if a sensor is currently attached to the tree. Used by
    /// `TreeService::delete` to populate `TreeDeleted { had_sensor }` so the
    /// cluster status aggregator can skip recalculation when the deleted tree
    /// did not contribute to it.
    pub fn had_sensor(&self) -> bool {
        self.sensor_id.is_some()
    }

    pub fn watering_status(&self) -> WateringStatus {
        self.watering_status
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Replaces all freely editable fields. `cluster_id` and `sensor_id` are
    /// not part of this update because they have their own
    /// transition-emitting methods (`move_to_cluster`, `attach_sensor`,
    /// `detach_sensor`). Only a `coordinate` change emits an event:
    /// `TreeCoordinateChanged` is consumed by the cluster recalc handler to
    /// update centroid and region. The other fields (species, tree_number,
    /// planting_year, description, provenance) have no subscribers, so they
    /// change silently.
    pub fn replace_details(
        &mut self,
        species: Species,
        tree_number: TreeNumber,
        planting_year: PlantingYear,
        coordinate: Coordinate,
        description: Option<String>,
        provenance: Provenance,
    ) -> Vec<DomainEvent> {
        let mut events = Vec::new();
        if self.coordinate != coordinate {
            events.push(DomainEvent::TreeCoordinateChanged {
                tree_id: self.id,
                cluster_id: self.cluster_id,
            });
        }
        self.species = species;
        self.tree_number = tree_number;
        self.planting_year = planting_year;
        self.coordinate = coordinate;
        self.description = description;
        self.provenance = provenance;
        events
    }

    pub fn move_to_cluster(&mut self, target: Option<Id<TreeCluster>>) -> Vec<DomainEvent> {
        if self.cluster_id == target {
            return vec![];
        }
        let from = self.cluster_id;
        self.cluster_id = target;
        vec![DomainEvent::TreeMovedBetweenClusters {
            tree_id: self.id,
            from,
            to: target,
        }]
    }

    pub fn attach_sensor(&mut self, sensor: SensorId) -> Vec<DomainEvent> {
        if self.sensor_id.as_ref() == Some(&sensor) {
            return vec![];
        }
        let mut events = Vec::new();
        if let Some(old) = self.sensor_id.take() {
            events.push(DomainEvent::TreeSensorDetached {
                tree_id: self.id,
                cluster_id: self.cluster_id,
                sensor_id: old,
            });
        }
        self.sensor_id = Some(sensor.clone());
        events.push(DomainEvent::TreeSensorAttached {
            tree_id: self.id,
            cluster_id: self.cluster_id,
            sensor_id: sensor,
        });
        events
    }

    /// Detaches the sensor and resets `watering_status` to
    /// [`WateringStatus::Unknown`].
    ///
    /// Once the sensor link is gone there is no data source to derive a status
    /// from, so the previous value is no longer meaningful. No
    /// `TreeWateringStatusChanged` event is emitted: the cluster aggregator
    /// already excludes sensorless trees from its average, so the state reset
    /// has no externally visible effect that `TreeSensorDetached` does not
    /// already cover.
    pub fn detach_sensor(&mut self) -> Vec<DomainEvent> {
        let Some(sensor_id) = self.sensor_id.take() else {
            return vec![];
        };
        self.watering_status = WateringStatus::Unknown;
        vec![DomainEvent::TreeSensorDetached {
            tree_id: self.id,
            cluster_id: self.cluster_id,
            sensor_id,
        }]
    }

    pub fn record_watering_status(&mut self, status: WateringStatus) -> Vec<DomainEvent> {
        if self.watering_status == status {
            return vec![];
        }
        self.watering_status = status;
        vec![DomainEvent::TreeWateringStatusChanged {
            tree_id: self.id,
            cluster_id: self.cluster_id,
            new_status: status,
        }]
    }

    pub fn mark_watered_at(&mut self, ts: DateTime<Utc>) {
        self.last_watered = Some(ts);
    }

    /// Derives a [`WateringStatus`] from three Watermark sensor readings
    /// (tension at depths 30/60/90 cm) and the tree's age. Mirrors the
    /// calibration the Go backend used — see
    /// [`watermark_calibration`] for the threshold tables and lifecycle
    /// rules. The worst per-depth score wins (Good < Moderate < Bad).
    ///
    /// This method is **Watermark-specific**. When other sensor families
    /// land, they will get sibling methods (e.g. `calculate_watering_status_from_volumetric`)
    /// rather than overloading this one.
    pub fn calculate_watering_status_from_watermarks(
        &self,
        watermarks: &[Watermark],
        today: DateTime<Utc>,
    ) -> Result<WateringStatus, TreeError> {
        let (w30, w60, w90) = watermark_calibration::sort_watermarks(watermarks)?;
        let lifetime = (today.year() as i64) - (self.planting_year.year() as i64);
        let tuning = watermark_calibration::PhaseTuning::for_year(lifetime)?;
        let scores = tuning.score(w30.centibar, w60.centibar, w90.centibar);

        let worst = scores.iter().copied().max().expect("scores has 3 elements");
        Ok(match worst {
            0 => WateringStatus::Good,
            1 => WateringStatus::Moderate,
            _ => WateringStatus::Bad,
        })
    }

    /// Derives a [`WateringStatus`] from volumetric soil-moisture readings
    /// (GES-1000 family). Sibling of
    /// [`calculate_watering_status_from_watermarks`]; the tree aggregate
    /// itself stays calibration-agnostic — the service layer picks which
    /// sibling to call based on the linked sensor's model.
    pub fn calculate_watering_status_from_volumetric(
        &self,
        readings: &[crate::sensor::data::VolumetricReading],
    ) -> Result<WateringStatus, TreeError> {
        volumetric_calibration::classify(readings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_tree() -> Tree {
        Tree {
            id: Id::new(1),
            planting_year: PlantingYear::new(2020).unwrap(),
            species: Species::new("Quercus robur").unwrap(),
            tree_number: TreeNumber::new("FL-001").unwrap(),
            coordinate: Coordinate::new(54.7937, 9.4469).unwrap(),
            description: None,
            last_watered: None,
            cluster_id: None,
            sensor_id: None,
            watering_status: WateringStatus::Unknown,
            provenance: Provenance::default(),
        }
    }

    #[test]
    fn species_rejects_empty() {
        assert_err!(Species::new(""));
    }

    #[test]
    fn species_accepts_valid() {
        assert_ok!(Species::new("Tilia cordata"));
    }

    #[test]
    fn tree_number_rejects_empty() {
        assert_err!(TreeNumber::new(""));
    }

    #[test]
    fn tree_number_accepts_valid() {
        assert_ok!(TreeNumber::new("FL-A-12"));
    }

    #[test]
    fn move_to_same_cluster_is_noop() {
        let mut t = fixed_tree();
        let events = t.move_to_cluster(None);
        assert!(t.cluster_id().is_none());
        assert!(events.is_empty());
    }

    #[test]
    fn move_to_new_cluster_emits_event_and_updates() {
        let mut t = fixed_tree();
        let target = Id::new(42);
        let events = t.move_to_cluster(Some(target));
        assert_eq!(t.cluster_id(), Some(target));
        assert_eq!(events.len(), 1);
        match &events[0] {
            DomainEvent::TreeMovedBetweenClusters { tree_id, from, to } => {
                assert_eq!(*tree_id, t.id);
                assert_eq!(*from, None);
                assert_eq!(*to, Some(target));
            }
            other => panic!("expected TreeMovedBetweenClusters, got {other:?}"),
        }
    }

    #[test]
    fn move_to_other_cluster_emits_event_with_from_and_to() {
        let mut t = fixed_tree();
        let a = Id::new(10);
        let b = Id::new(20);
        t.move_to_cluster(Some(a));
        let events = t.move_to_cluster(Some(b));
        assert_eq!(t.cluster_id(), Some(b));
        match &events[0] {
            DomainEvent::TreeMovedBetweenClusters { from, to, .. } => {
                assert_eq!(*from, Some(a));
                assert_eq!(*to, Some(b));
            }
            other => panic!("expected TreeMovedBetweenClusters, got {other:?}"),
        }
    }

    #[test]
    fn move_out_of_cluster_emits_event_with_to_none() {
        let mut t = fixed_tree();
        let a = Id::new(10);
        t.move_to_cluster(Some(a));
        let events = t.move_to_cluster(None);
        assert!(t.cluster_id().is_none());
        match &events[0] {
            DomainEvent::TreeMovedBetweenClusters { from, to, .. } => {
                assert_eq!(*from, Some(a));
                assert_eq!(*to, None);
            }
            other => panic!("expected TreeMovedBetweenClusters, got {other:?}"),
        }
    }

    #[test]
    fn replace_details_with_new_coordinate_emits_event() {
        let mut t = fixed_tree();
        let new_coord = Coordinate::new(54.5, 9.5).unwrap();
        let events = t.replace_details(
            t.species.clone(),
            t.tree_number.clone(),
            t.planting_year,
            new_coord,
            None,
            Provenance::default(),
        );
        assert_eq!(t.coordinate, new_coord);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::TreeCoordinateChanged { .. }
        ));
    }

    #[test]
    fn replace_details_with_same_coordinate_emits_no_event() {
        let mut t = fixed_tree();
        let same_coord = t.coordinate;
        let events = t.replace_details(
            Species::new("Different species").unwrap(),
            t.tree_number.clone(),
            t.planting_year,
            same_coord,
            Some("new desc".into()),
            Provenance::default(),
        );
        assert_eq!(t.coordinate, same_coord);
        assert!(events.is_empty(), "non-coordinate fields don't emit events");
    }

    #[test]
    fn detach_sensor_emits_detached_event_and_resets_status() {
        let mut t = fixed_tree();
        let sensor = SensorId::new("eui-deadbeef").unwrap();
        let _ = t.attach_sensor(sensor.clone());
        let _ = t.record_watering_status(WateringStatus::Good);
        let events = t.detach_sensor();
        assert!(t.sensor_id().is_none());
        assert_eq!(t.watering_status(), WateringStatus::Unknown);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::TreeSensorDetached { .. }));
    }

    #[test]
    fn detach_sensor_when_none_is_noop() {
        let mut t = fixed_tree();
        let events = t.detach_sensor();
        assert!(t.sensor_id().is_none());
        assert!(events.is_empty());
    }

    #[test]
    fn attach_sensor_replaces_existing_emits_detach_then_attach() {
        let mut t = fixed_tree();
        let s1 = SensorId::new("eui-aaaa").unwrap();
        let s2 = SensorId::new("eui-bbbb").unwrap();
        let _ = t.attach_sensor(s1.clone());
        let events = t.attach_sensor(s2.clone());
        assert_eq!(t.sensor_id(), Some(&s2));
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], DomainEvent::TreeSensorDetached { .. }));
        assert!(matches!(events[1], DomainEvent::TreeSensorAttached { .. }));
    }

    #[test]
    fn attach_sensor_first_time_emits_only_attach() {
        let mut t = fixed_tree();
        let s = SensorId::new("eui-deadbeef").unwrap();
        let events = t.attach_sensor(s.clone());
        assert_eq!(t.sensor_id(), Some(&s));
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], DomainEvent::TreeSensorAttached { .. }));
    }

    #[test]
    fn attach_sensor_same_id_is_noop() {
        let mut t = fixed_tree();
        let s = SensorId::new("eui-deadbeef").unwrap();
        let _ = t.attach_sensor(s.clone());
        let events = t.attach_sensor(s);
        assert!(events.is_empty());
    }

    #[test]
    fn record_watering_status_change_emits_event() {
        let mut t = fixed_tree();
        let events = t.record_watering_status(WateringStatus::Good);
        assert_eq!(t.watering_status(), WateringStatus::Good);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            DomainEvent::TreeWateringStatusChanged {
                new_status: WateringStatus::Good,
                ..
            }
        ));
    }

    #[test]
    fn record_watering_status_same_value_is_noop() {
        let mut t = fixed_tree();
        let _ = t.record_watering_status(WateringStatus::Good);
        let events = t.record_watering_status(WateringStatus::Good);
        assert!(events.is_empty());
    }

    #[test]
    fn had_sensor_reflects_current_state() {
        let mut t = fixed_tree();
        assert!(!t.had_sensor());
        let _ = t.attach_sensor(SensorId::new("eui-deadbeef").unwrap());
        assert!(t.had_sensor());
        let _ = t.detach_sensor();
        assert!(!t.had_sensor());
    }

    #[test]
    fn mark_watered_at_sets_timestamp() {
        let mut t = fixed_tree();
        let ts = Utc::now();
        t.mark_watered_at(ts);
        assert_eq!(t.last_watered, Some(ts));
    }

    fn wm(depth: i32, centibar: i32) -> Watermark {
        Watermark {
            depth,
            resistance: 0,
            centibar,
        }
    }

    fn tree_planted_in(year: u32) -> Tree {
        let mut t = fixed_tree();
        t.planting_year = PlantingYear::reconstitute(year);
        t
    }

    fn jan_first(year: i32) -> DateTime<Utc> {
        use chrono::TimeZone;
        Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn watering_status_year0_and_1_share_default_thresholds() {
        let t = tree_planted_in(2024);
        let today = jan_first(2024);
        let dry = vec![wm(30, 50), wm(60, 50), wm(90, 50)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&dry, today)
                .unwrap(),
            WateringStatus::Bad
        );
        let wet = vec![wm(30, 5), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&wet, today)
                .unwrap(),
            WateringStatus::Good
        );
        let mod_ = vec![wm(30, 28), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&mod_, today)
                .unwrap(),
            WateringStatus::Moderate
        );
    }

    #[test]
    fn watering_status_year2_uses_wider_depth30_band() {
        let t = tree_planted_in(2022);
        let today = jan_first(2024);
        let just_under_y2_d30_lower = vec![wm(30, 60), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&just_under_y2_d30_lower, today)
                .unwrap(),
            WateringStatus::Good,
            "centibar=60 is below 62 lower bound for year-2 depth 30"
        );
        let in_y2_moderate = vec![wm(30, 70), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&in_y2_moderate, today)
                .unwrap(),
            WateringStatus::Moderate
        );
    }

    #[test]
    fn watering_status_year3_has_no_moderate() {
        let t = tree_planted_in(2021);
        let today = jan_first(2024);
        let high_d30 = vec![wm(30, 200), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&high_d30, today)
                .unwrap(),
            WateringStatus::Good,
            "year-3 depth-30 lower bound is 1585; 200 < 1585 is Good"
        );
        let bad_d60 = vec![wm(30, 100), wm(60, 100), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&bad_d60, today)
                .unwrap(),
            WateringStatus::Bad,
            "year-3 depth-60 lower bound is 80; 100 >= 80 with no moderate band → Bad"
        );
    }

    #[test]
    fn watering_status_beyond_year3_rejects() {
        let t = tree_planted_in(2020);
        let today = jan_first(2025);
        let any = vec![wm(30, 5), wm(60, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&any, today)
                .unwrap_err(),
            TreeError::BeyondMonitoring
        );
    }

    #[test]
    fn watering_status_rejects_malformed_watermark_set() {
        let t = tree_planted_in(2024);
        let today = jan_first(2024);
        let only_two = vec![wm(30, 5), wm(60, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&only_two, today)
                .unwrap_err(),
            TreeError::MalformedWatermarks
        );
        let wrong_depth = vec![wm(30, 5), wm(45, 5), wm(90, 5)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&wrong_depth, today)
                .unwrap_err(),
            TreeError::MalformedWatermarks
        );
    }

    #[test]
    fn watering_status_takes_worst_among_depths() {
        let t = tree_planted_in(2024);
        let today = jan_first(2024);
        let mixed = vec![wm(30, 5), wm(60, 28), wm(90, 50)];
        assert_eq!(
            t.calculate_watering_status_from_watermarks(&mixed, today)
                .unwrap(),
            WateringStatus::Bad,
            "any depth in Bad band makes the whole reading Bad"
        );
    }
}
