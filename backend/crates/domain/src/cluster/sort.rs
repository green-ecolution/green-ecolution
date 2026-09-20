//! Sort options for the tree cluster list.

use crate::shared::sort::SortDirection;

/// The sortable columns of the cluster list.
///
/// `as_sql_key` is the literal the repository's sort-column table is keyed by.
/// Because the set is closed, no caller-supplied text ever reaches SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClusterSortField {
    #[default]
    Name,
    Moisture,
    Trees,
    LastWatered,
}

impl ClusterSortField {
    pub fn as_sql_key(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Moisture => "moisture",
            Self::Trees => "trees",
            Self::LastWatered => "last_watered",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClusterSort {
    pub field: ClusterSortField,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_keys_are_exactly_the_contracted_set() {
        // This string set is a contract with CLUSTER_SORT_COLUMNS in
        // pg_cluster.rs; changing it requires a matching change there.
        let keys: Vec<&str> = [
            ClusterSortField::Name,
            ClusterSortField::Moisture,
            ClusterSortField::Trees,
            ClusterSortField::LastWatered,
        ]
        .into_iter()
        .map(ClusterSortField::as_sql_key)
        .collect();

        assert_eq!(keys, vec!["name", "moisture", "trees", "last_watered"]);
    }

    #[test]
    fn defaults_to_name_ascending() {
        let sort = ClusterSort::default();
        assert_eq!(sort.field, ClusterSortField::Name);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
