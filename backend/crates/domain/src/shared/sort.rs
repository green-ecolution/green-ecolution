//! Sort direction, shared by every list query.

use std::{fmt, str::FromStr};

use crate::shared::error::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn is_descending(self) -> bool {
        matches!(self, Self::Descending)
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ascending => f.write_str("asc"),
            Self::Descending => f.write_str("desc"),
        }
    }
}

impl FromStr for SortDirection {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Ascending),
            "desc" => Ok(Self::Descending),
            other => Err(ValidationError::InvalidFormat {
                field: "sort.order",
                reason: format!("unknown sort order `{other}`"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_directions() {
        assert_eq!(
            "asc".parse::<SortDirection>().unwrap(),
            SortDirection::Ascending
        );
        assert_eq!(
            "desc".parse::<SortDirection>().unwrap(),
            SortDirection::Descending
        );
    }

    #[test]
    fn rejects_unknown_direction() {
        assert!("sideways".parse::<SortDirection>().is_err());
    }

    #[test]
    fn round_trips_through_display() {
        for direction in [SortDirection::Ascending, SortDirection::Descending] {
            assert_eq!(
                direction.to_string().parse::<SortDirection>().unwrap(),
                direction
            );
        }
    }

    #[test]
    fn defaults_to_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
        assert!(!SortDirection::default().is_descending());
    }
}
