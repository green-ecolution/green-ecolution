use chrono::{Datelike, Utc};

use crate::shared::error::ValidationError;

/// Calendar year in which a tree was planted; must not be in the future.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantingYear(u32);

impl PlantingYear {
    pub fn new(year: u32) -> Result<Self, ValidationError> {
        let current_year = Utc::now().year() as u32;
        if year > current_year {
            return Err(ValidationError::OutOfRange {
                field: "tree.planting_year",
                min: 0.0,
                max: current_year as f64,
                got: year as f64,
            });
        }
        Ok(Self(year))
    }

    #[allow(dead_code)]
    pub fn reconstitute(year: u32) -> Self {
        Self(year)
    }

    pub fn year(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for PlantingYear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn rejects_future_year() {
        let next_year = Utc::now().year() as u32 + 1;
        assert_err!(PlantingYear::new(next_year));
    }

    #[test]
    fn accepts_current_year() {
        let current = Utc::now().year() as u32;
        assert_ok!(PlantingYear::new(current));
    }

    #[test]
    fn accepts_past_year() {
        assert_ok!(PlantingYear::new(2000));
    }
}
