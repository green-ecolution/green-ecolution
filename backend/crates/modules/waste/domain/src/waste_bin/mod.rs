//! The `WasteBin` aggregate: a container with a fill level sensor.
//!
//! Invariants:
//! - The fill level is derived from the sensor distance and the bin depth, never set directly.
//! - A reading that no container can produce changes nothing.
//! - A reading older than the recorded one, and an emptying older than the recorded one, are ignored.
//! - Emptying after the last measurement wins over that measurement.
//! - The owning organization changes only through `transfer_to`.

mod repository;
mod snapshot;
mod view;

use chrono::{DateTime, Utc};

use domain::{
    Id,
    organization::Organization,
    shared::{coordinates::Coordinate, error::ValidationError, provenance::Provenance},
};

pub use repository::{WasteBinReader, WasteBinWriter};
#[doc(hidden)]
pub use snapshot::WasteBinSnapshot;
pub use view::WasteBinView;

use crate::fill::{FillMeasurement, FillState, derive_fill_state, fill_from_distance};
use crate::settings::FullThreshold;
use crate::values::{BinDepth, BinVolume, WasteBinNumber};

#[derive(Debug, Clone)]
pub struct WasteBin {
    pub id: Id<WasteBin>,
    pub number: WasteBinNumber,
    pub coordinate: Coordinate,
    pub volume: BinVolume,
    pub depth: BinDepth,
    organization_id: Id<Organization>,
    provenance: Provenance,
    fill: Option<FillMeasurement>,
    last_emptied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct WasteBinDraft {
    pub number: WasteBinNumber,
    pub coordinate: Coordinate,
    pub volume: BinVolume,
    pub depth: BinDepth,
    pub organization_id: Id<Organization>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone)]
pub struct WasteBinUpdate {
    pub number: WasteBinNumber,
    pub coordinate: Coordinate,
    pub volume: BinVolume,
    pub depth: BinDepth,
}

#[derive(Debug, Clone, Default)]
pub struct WasteBinSearchQuery {
    pub number: Option<String>,
    pub organization_id: Option<Id<Organization>>,
    pub fill: Option<FillFilter>,
}

/// Narrows a search by the current fill level. All set conditions apply together.
///
/// The bounds depend on the organization's rules (full threshold, staleness
/// limit), so they come per organization; a bin of an organization without
/// an entry never matches. The flags depend only on the asked-for state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FillFilter {
    pub bounds: Vec<OrganizationFillBounds>,
    pub require_measurement: bool,
    pub not_emptied_since_measurement: bool,
    pub emptied_after_measurement: bool,
}

/// The rule-dependent part of a [`FillFilter`] for one organization's bins.
#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationFillBounds {
    pub organization_id: Id<Organization>,
    /// Inclusive lower bound of the fill fraction.
    pub min_fraction: Option<f64>,
    /// Exclusive upper bound of the fill fraction.
    pub max_fraction: Option<f64>,
    /// Inclusive lower bound of the measurement time.
    pub measured_after: Option<DateTime<Utc>>,
    /// Matches bins without a reading or with a reading strictly before this instant.
    pub stale_before: Option<DateTime<Utc>>,
}

impl WasteBin {
    #[doc(hidden)]
    pub fn reconstitute(snap: WasteBinSnapshot) -> Result<Self, ValidationError> {
        let fill = match (snap.fill_fraction, snap.fill_measured_at) {
            (Some(fraction), Some(measured_at)) => Some(FillMeasurement {
                fraction,
                measured_at,
            }),
            _ => None,
        };
        Ok(Self {
            id: Id::new(snap.id),
            number: WasteBinNumber::new(snap.number)?,
            coordinate: Coordinate::new(snap.latitude, snap.longitude)?,
            volume: BinVolume::new(snap.volume_liters)?,
            depth: BinDepth::new(i64::from(snap.depth_cm))?,
            organization_id: Id::new(snap.organization_id),
            provenance: Provenance::reconstitute(snap.provider, snap.additional_informations),
            fill,
            last_emptied_at: snap.last_emptied_at,
        })
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn fill(&self) -> Option<FillMeasurement> {
        self.fill
    }

    pub fn last_emptied_at(&self) -> Option<DateTime<Utc>> {
        self.last_emptied_at
    }

    /// Ignores a reading older than the one already recorded.
    pub fn record_fill_level(&mut self, distance_cm: f64, measured_at: DateTime<Utc>) -> bool {
        if self.fill.is_some_and(|f| measured_at <= f.measured_at) {
            return false;
        }
        let Some(fraction) = fill_from_distance(distance_cm, self.depth) else {
            return false;
        };
        self.fill = Some(FillMeasurement {
            fraction,
            measured_at,
        });
        true
    }

    pub fn mark_emptied(&mut self, at: DateTime<Utc>) {
        if self.last_emptied_at.is_none_or(|current| at > current) {
            self.last_emptied_at = Some(at);
        }
    }

    pub fn relocate(&mut self, coordinate: Coordinate) {
        self.coordinate = coordinate;
    }

    pub fn replace_details(&mut self, update: WasteBinUpdate) {
        self.number = update.number;
        self.coordinate = update.coordinate;
        self.volume = update.volume;
        self.depth = update.depth;
    }

    pub fn transfer_to(&mut self, organization: Id<Organization>) {
        self.organization_id = organization;
    }

    pub fn fill_state(
        &self,
        threshold: FullThreshold,
        now: DateTime<Utc>,
        stale_after: chrono::Duration,
    ) -> FillState {
        derive_fill_state(self.fill, self.last_emptied_at, threshold, now, stale_after)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};
    use uuid::Uuid;

    use super::*;

    fn at(hours: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap() + Duration::hours(hours)
    }

    fn snapshot(depth_cm: i32) -> WasteBinSnapshot {
        WasteBinSnapshot {
            id: Uuid::now_v7(),
            number: "B-1".into(),
            latitude: 54.79,
            longitude: 9.44,
            volume_liters: 1100.0,
            depth_cm,
            organization_id: Uuid::now_v7(),
            provider: None,
            additional_informations: None,
            fill_fraction: None,
            fill_measured_at: None,
            last_emptied_at: None,
        }
    }

    fn bin_with_depth(depth_cm: i32) -> WasteBin {
        WasteBin::reconstitute(snapshot(depth_cm)).unwrap()
    }

    #[test]
    fn a_reading_sets_the_fill_from_the_depth() {
        let mut bin = bin_with_depth(120);
        assert!(bin.record_fill_level(30.0, at(0)));
        assert_eq!(bin.fill().unwrap().fraction, 0.75);
    }

    #[test]
    fn an_older_reading_does_not_overwrite_a_newer_one() {
        let mut bin = bin_with_depth(100);
        bin.record_fill_level(10.0, at(2));
        assert!(!bin.record_fill_level(90.0, at(1)));
        assert_eq!(bin.fill().unwrap().fraction, 0.9);
    }

    #[test]
    fn an_implausible_reading_changes_nothing() {
        let mut bin = bin_with_depth(100);
        assert!(!bin.record_fill_level(-5.0, at(0)));
        assert!(bin.fill().is_none());
    }

    #[test]
    fn emptying_is_recorded_and_wins_over_the_last_reading() {
        let mut bin = bin_with_depth(100);
        bin.record_fill_level(5.0, at(0));
        bin.mark_emptied(at(1));
        assert_eq!(
            bin.fill_state(FullThreshold::new(80).unwrap(), at(2), Duration::hours(6)),
            FillState::Empty
        );
    }

    #[test]
    fn an_earlier_emptying_than_the_recorded_one_is_ignored() {
        let mut bin = bin_with_depth(100);
        bin.mark_emptied(at(3));
        bin.mark_emptied(at(1));
        assert_eq!(bin.last_emptied_at(), Some(at(3)));
    }

    #[test]
    fn replace_details_keeps_fill_and_organization() {
        let mut bin = bin_with_depth(100);
        bin.record_fill_level(20.0, at(0));
        bin.mark_emptied(at(1));
        let (org, fill, emptied) = (bin.organization_id(), bin.fill(), bin.last_emptied_at());

        let update = WasteBinUpdate {
            number: WasteBinNumber::new("B-2").unwrap(),
            coordinate: Coordinate::new(54.5, 9.5).unwrap(),
            volume: BinVolume::new(240.0).unwrap(),
            depth: BinDepth::new(90).unwrap(),
        };
        bin.replace_details(update.clone());

        assert_eq!(bin.number, update.number);
        assert_eq!(bin.coordinate, update.coordinate);
        assert_eq!(bin.volume, update.volume);
        assert_eq!(bin.depth, update.depth);
        assert_eq!(bin.organization_id(), org);
        assert_eq!(bin.fill(), fill);
        assert_eq!(bin.last_emptied_at(), emptied);
    }

    #[test]
    fn relocate_moves_the_coordinate_only() {
        let mut bin = bin_with_depth(100);
        bin.record_fill_level(20.0, at(0));
        bin.mark_emptied(at(1));
        let before = bin.clone();
        let target = Coordinate::new(54.5, 9.5).unwrap();

        bin.relocate(target);

        assert_eq!(bin.coordinate, target);
        assert_ne!(before.coordinate, target);
        assert_eq!(bin.number, before.number);
        assert_eq!(bin.volume, before.volume);
        assert_eq!(bin.depth, before.depth);
        assert_eq!(bin.organization_id(), before.organization_id());
        assert_eq!(bin.fill(), before.fill());
        assert_eq!(bin.last_emptied_at(), before.last_emptied_at());
    }

    #[test]
    fn transfer_moves_the_organization_only() {
        let mut bin = bin_with_depth(100);
        bin.record_fill_level(20.0, at(0));
        let before = bin.clone();
        let target = Id::<Organization>::new(Uuid::now_v7());

        bin.transfer_to(target);

        assert_eq!(bin.organization_id(), target);
        assert_ne!(before.organization_id(), target);
        assert_eq!(bin.id, before.id);
        assert_eq!(bin.number, before.number);
        assert_eq!(bin.coordinate, before.coordinate);
        assert_eq!(bin.volume, before.volume);
        assert_eq!(bin.depth, before.depth);
        assert_eq!(bin.fill(), before.fill());
        assert_eq!(bin.last_emptied_at(), before.last_emptied_at());
    }
}
