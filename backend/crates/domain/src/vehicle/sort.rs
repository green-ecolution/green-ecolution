//! Sort options for the vehicle list.

use crate::shared::sort::SortDirection;

/// The sortable columns of the vehicle list.
///
/// `as_sql_key` is the literal the repository's sort-column table is keyed by.
/// Because the set is closed, no caller-supplied text ever reaches SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VehicleSortField {
    #[default]
    NumberPlate,
    WaterCapacity,
    Model,
    Type,
}

impl VehicleSortField {
    pub fn as_sql_key(self) -> &'static str {
        match self {
            Self::NumberPlate => "number_plate",
            Self::WaterCapacity => "water_capacity",
            Self::Model => "model",
            Self::Type => "type",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            VehicleSortField::WaterCapacity,
            VehicleSortField::Model,
            VehicleSortField::Type,
        ]
        .into_iter()
        .map(VehicleSortField::as_sql_key)
        .collect();

        assert_eq!(
            keys,
            vec!["number_plate", "water_capacity", "model", "type"]
        );
    }

    #[test]
    fn defaults_to_number_plate_ascending() {
        let sort = VehicleSort::default();
        assert_eq!(sort.field, VehicleSortField::NumberPlate);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
