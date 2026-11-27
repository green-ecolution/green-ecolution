//! Sort options for the vehicle list.

use crate::{capacity::CapacityKind, shared::sort::SortDirection};

/// The sortable columns of the vehicle list.
///
/// `as_sql_key` is the literal the repository's sort-column table is keyed by.
/// Because the set is closed, no caller-supplied text ever reaches SQL; the
/// capacity kind travels as a bound value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum VehicleSortField {
    #[default]
    NumberPlate,
    /// The vehicle's amount of this kind, 0 where it has none.
    Capacity(CapacityKind),
    Model,
    Type,
}

impl VehicleSortField {
    pub fn as_sql_key(&self) -> &'static str {
        match self {
            Self::NumberPlate => "number_plate",
            Self::Capacity(_) => "capacity",
            Self::Model => "model",
            Self::Type => "type",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VehicleSort {
    pub field: VehicleSortField,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_keys_are_exactly_the_contracted_set() {
        // This string set is a contract with VEHICLE_SORT_COLUMNS in
        // pg_vehicle.rs; changing it requires a matching change there.
        let keys: Vec<&str> = [
            VehicleSortField::NumberPlate,
            VehicleSortField::Capacity(crate::capacity::CapacityKind::new("water").unwrap()),
            VehicleSortField::Model,
            VehicleSortField::Type,
        ]
        .iter()
        .map(VehicleSortField::as_sql_key)
        .collect();

        assert_eq!(keys, vec!["number_plate", "capacity", "model", "type"]);
    }

    #[test]
    fn defaults_to_number_plate_ascending() {
        let sort = VehicleSort::default();
        assert_eq!(sort.field, VehicleSortField::NumberPlate);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
