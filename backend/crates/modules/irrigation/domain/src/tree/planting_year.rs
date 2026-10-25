use chrono::{DateTime, Datelike, Utc};

use domain::shared::error::ValidationError;

/// Oldest plausible planting year. Anything below is a typo (a bare `25` meant
/// as 2025) rather than a record, and the OpenAPI schema already advertises it.
pub const MIN_PLANTING_YEAR: u32 = 1900;

/// Upper bound, matching the range the OpenAPI schema has always advertised.
/// Future years are valid: plantings are scheduled before they happen.
pub const MAX_PLANTING_YEAR: u32 = 2100;

/// Calendar year in which a tree was planted or is scheduled to be planted;
/// must lie between [`MIN_PLANTING_YEAR`] and [`MAX_PLANTING_YEAR`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlantingYear(u32);

impl PlantingYear {
    pub fn new(year: u32) -> Result<Self, ValidationError> {
        if !(MIN_PLANTING_YEAR..=MAX_PLANTING_YEAR).contains(&year) {
            return Err(ValidationError::OutOfRange {
                field: "tree.planting_year",
                min: MIN_PLANTING_YEAR as f64,
                max: MAX_PLANTING_YEAR as f64,
                got: year as f64,
            });
        }
        Ok(Self(year))
    }

    pub fn reconstitute(year: u32) -> Self {
        Self(year)
    }

    pub fn year(&self) -> u32 {
        self.0
    }

    /// Whether the planting still lies ahead. The single definition of "not
    /// planted yet" — the status calculations and the sensor rule share it.
    pub fn is_future(&self, today: DateTime<Utc>) -> bool {
        (self.0 as i64) > (today.year() as i64)
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
    use chrono::{Datelike, Utc};
    use claims::{assert_err, assert_ok};

    // Plantings are scheduled ahead, so a year that has not arrived yet is a
    // legitimate record rather than a mistake.
    #[test]
    fn accepts_year_in_the_near_future() {
        let next_year = Utc::now().year() as u32 + 1;
        assert_ok!(PlantingYear::new(next_year));
    }

    #[test]
    fn accepts_upper_bound() {
        assert_ok!(PlantingYear::new(MAX_PLANTING_YEAR));
    }

    #[test]
    fn rejects_year_after_upper_bound() {
        assert_err!(PlantingYear::new(MAX_PLANTING_YEAR + 1));
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

    #[test]
    fn rejects_two_digit_year() {
        assert_err!(PlantingYear::new(25));
    }

    #[test]
    fn rejects_year_before_lower_bound() {
        assert_err!(PlantingYear::new(MIN_PLANTING_YEAR - 1));
    }

    #[test]
    fn accepts_lower_bound() {
        assert_ok!(PlantingYear::new(MIN_PLANTING_YEAR));
    }

    #[test]
    fn reconstitute_bypasses_lower_bound() {
        assert_eq!(PlantingYear::reconstitute(25).year(), 25);
    }
}
