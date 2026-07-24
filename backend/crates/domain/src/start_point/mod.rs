//! StartPoint aggregate — a named routing location a watering route starts and
//! returns from; some are also water refill points (`watering_point`).
//!
//! Follows the `Region` pattern: aggregate + `StartPointDraft` for creation, a
//! `pub(crate)` snapshot for DB rehydration, and a `StartPointReader` /
//! `StartPointWriter` trait split. No view type — the aggregate is small enough
//! to serve as its own read model.
//!
//! The "exactly one default" rule spans rows and therefore lives at the
//! service/DB level (a partial unique index plus a single-statement update in
//! the writer), not inside this aggregate.

pub mod error;
pub mod repository;
pub mod snapshot;

use crate::{Id, events::DomainEvent, organization::Organization, shared::coordinates::Coordinate};

pub use error::StartPointError;
pub use repository::{StartPointReader, StartPointWriter};
#[doc(hidden)]
pub use snapshot::StartPointSnapshot;

crate::newtype_nonempty! {
    /// Start point name, 1–255 characters after trimming.
    StartPointName, "start_point.name", 1, 255
}

#[derive(Debug, Clone, PartialEq)]
pub struct StartPoint {
    pub id: Id<StartPoint>,
    pub name: StartPointName,
    pub coordinate: Coordinate,
    is_default: bool,
    watering_point: bool,
    organization_id: Id<Organization>,
}

/// Input for creating a new [`StartPoint`]. New points are never default;
/// promotion happens through [`StartPointWriter::set_default`].
#[derive(Debug, Clone)]
pub struct StartPointDraft {
    pub name: StartPointName,
    pub coordinate: Coordinate,
    pub watering_point: bool,
    pub organization_id: Id<Organization>,
}

/// Replacement input for editing a [`StartPoint`]. Does not touch `is_default`.
#[derive(Debug, Clone)]
pub struct StartPointUpdate {
    pub name: StartPointName,
    pub coordinate: Coordinate,
    pub watering_point: bool,
}

impl StartPoint {
    #[doc(hidden)]
    pub fn reconstitute(snap: StartPointSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            name: StartPointName::reconstitute(snap.name),
            coordinate: Coordinate::new(snap.latitude, snap.longitude)
                .expect("persisted start point coordinate must be valid"),
            is_default: snap.is_default,
            watering_point: snap.watering_point,
            organization_id: Id::new(snap.organization_id),
        }
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn watering_point(&self) -> bool {
        self.watering_point
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn rename(&mut self, new_name: StartPointName) {
        if self.name == new_name {
            return;
        }
        self.name = new_name;
    }

    pub fn relocate(&mut self, coordinate: Coordinate) {
        self.coordinate = coordinate;
    }

    pub fn set_watering_point(&mut self, value: bool) {
        self.watering_point = value;
    }

    /// Reassigns the start point to `target`'s organization. No-op if it
    /// already belongs there. Also clears `is_default`: the target org's own
    /// default (if any) must stay untouched, and carrying `is_default = true`
    /// across the transfer could otherwise collide with it under
    /// `depots_single_default_per_org`. Promotion in the new organization is
    /// a separate, explicit call to `StartPointWriter::set_default`. No
    /// domain-event subscribers exist for this aggregate, so the
    /// `Vec<DomainEvent>` return type only mirrors the shared `transfer_to`
    /// shape used by other aggregates.
    pub fn transfer_to(&mut self, target: Id<Organization>) -> Vec<DomainEvent> {
        if self.organization_id == target {
            return vec![];
        }
        self.organization_id = target;
        self.is_default = false;
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed() -> StartPoint {
        StartPoint {
            id: Id::new_v7(),
            name: StartPointName::new("Betriebshof").unwrap(),
            coordinate: Coordinate::new(54.768, 9.434).unwrap(),
            is_default: true,
            watering_point: true,
            organization_id: Id::new_v7(),
        }
    }

    #[test]
    fn name_rejects_empty() {
        assert_err!(StartPointName::new(""));
    }

    #[test]
    fn name_accepts_valid() {
        assert_ok!(StartPointName::new("Klärwerk Kielseng"));
    }

    #[test]
    fn rename_to_same_is_noop() {
        let mut sp = fixed();
        sp.rename(StartPointName::new("Betriebshof").unwrap());
        assert_eq!(sp.name.as_str(), "Betriebshof");
    }

    #[test]
    fn rename_changes_name() {
        let mut sp = fixed();
        sp.rename(StartPointName::new("Nord").unwrap());
        assert_eq!(sp.name.as_str(), "Nord");
    }

    #[test]
    fn relocate_changes_coordinate() {
        let mut sp = fixed();
        sp.relocate(Coordinate::new(54.805, 9.447).unwrap());
        assert!((sp.coordinate.latitude() - 54.805).abs() < 1e-9);
    }

    #[test]
    fn set_watering_point_toggles() {
        let mut sp = fixed();
        sp.set_watering_point(false);
        assert!(!sp.watering_point());
    }

    #[test]
    fn transfer_to_same_org_is_noop() {
        let mut sp = fixed();
        let same = sp.organization_id();
        let was_default = sp.is_default();
        let events = sp.transfer_to(same);
        assert_eq!(sp.organization_id(), same);
        assert_eq!(
            sp.is_default(),
            was_default,
            "no-op must not reset is_default"
        );
        assert!(events.is_empty());
    }

    #[test]
    fn transfer_to_new_org_updates_field_and_resets_default() {
        let mut sp = fixed(); // is_default: true
        let target: Id<Organization> = Id::new_v7();
        let events = sp.transfer_to(target);
        assert_eq!(sp.organization_id(), target);
        assert!(
            !sp.is_default(),
            "transfer must clear is_default to avoid a collision with the target org's own default"
        );
        assert!(
            events.is_empty(),
            "StartPoint has no domain-event subscribers"
        );
    }
}
