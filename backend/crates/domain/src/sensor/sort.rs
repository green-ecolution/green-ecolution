//! Sort options for the sensor list.

use crate::shared::sort::SortDirection;

/// The sortable columns of the sensor list.
///
/// `as_sql_key` is the literal the repository's sort-column table is keyed by.
/// Because the set is closed, no caller-supplied text ever reaches SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SensorSortField {
    #[default]
    Id,
    LastReading,
    CreatedAt,
}

impl SensorSortField {
    pub fn as_sql_key(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::LastReading => "last_reading",
            Self::CreatedAt => "created_at",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SensorSort {
    pub field: SensorSortField,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_keys_are_exactly_the_contracted_set() {
        // This string set is a contract with SENSOR_SORT_COLUMNS in
        // pg_sensor.rs; changing it requires a matching change there.
        let keys: Vec<&str> = [
            SensorSortField::Id,
            SensorSortField::LastReading,
            SensorSortField::CreatedAt,
        ]
        .into_iter()
        .map(SensorSortField::as_sql_key)
        .collect();

        assert_eq!(keys, vec!["id", "last_reading", "created_at"]);
    }

    #[test]
    fn defaults_to_id_ascending() {
        let sort = SensorSort::default();
        assert_eq!(sort.field, SensorSortField::Id);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
