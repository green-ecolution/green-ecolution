//! Sort options for the tree list.

use std::{fmt, str::FromStr};

use crate::shared::{error::ValidationError, sort::SortDirection};

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

impl fmt::Display for TreeSortField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_sql_key())
    }
}

impl FromStr for TreeSortField {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "number" => Ok(Self::Number),
            "species" => Ok(Self::Species),
            "status" => Ok(Self::Status),
            "planting_year" => Ok(Self::PlantingYear),
            "last_watered" => Ok(Self::LastWatered),
            "cluster" => Ok(Self::Cluster),
            other => Err(ValidationError::InvalidFormat {
                field: "tree.sort",
                reason: format!("unknown sort field `{other}`"),
            }),
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
    fn every_field_round_trips_through_its_sql_key() {
        for field in [
            TreeSortField::Number,
            TreeSortField::Species,
            TreeSortField::Status,
            TreeSortField::PlantingYear,
            TreeSortField::LastWatered,
            TreeSortField::Cluster,
        ] {
            assert_eq!(field.as_sql_key().parse::<TreeSortField>().unwrap(), field);
        }
    }

    #[test]
    fn rejects_unknown_field() {
        assert!("height".parse::<TreeSortField>().is_err());
    }

    #[test]
    fn defaults_to_number_ascending() {
        let sort = TreeSort::default();
        assert_eq!(sort.field, TreeSortField::Number);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
