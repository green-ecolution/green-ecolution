//! Sort options for the tree list.

use crate::shared::sort::SortDirection;

/// The sortable columns of the tree list.
///
/// `as_sql_key` is the literal the repository's `ORDER BY` compares against.
/// Because the set is closed, no caller-supplied text ever reaches SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeSortField {
    #[default]
    Number,
    Species,
    Status,
    PlantingYear,
    LastWatered,
    Cluster,
}

impl TreeSortField {
    pub fn as_sql_key(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Species => "species",
            Self::Status => "status",
            Self::PlantingYear => "planting_year",
            Self::LastWatered => "last_watered",
            Self::Cluster => "cluster",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TreeSort {
    pub field: TreeSortField,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_keys_are_exactly_the_contracted_set() {
        // This string set is a contract with the SQL ORDER BY clause in
        // pg_tree.rs; changing it requires a matching migration there.
        let keys: Vec<&str> = [
            TreeSortField::Number,
            TreeSortField::Species,
            TreeSortField::Status,
            TreeSortField::PlantingYear,
            TreeSortField::LastWatered,
            TreeSortField::Cluster,
        ]
        .into_iter()
        .map(TreeSortField::as_sql_key)
        .collect();

        assert_eq!(
            keys,
            vec![
                "number",
                "species",
                "status",
                "planting_year",
                "last_watered",
                "cluster",
            ]
        );
    }

    #[test]
    fn defaults_to_number_ascending() {
        let sort = TreeSort::default();
        assert_eq!(sort.field, TreeSortField::Number);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
