use std::fmt::Display;

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate(f64, f64);

impl Coordinate {
    pub fn new(lat: f64, lng: f64) -> Result<Coordinate, DomainError> {
        if lat < -90.0 || lat > 90.0 {
            return Err(DomainError::InvalidLatitude(lat));
        }

        if lng < -180.0 || lng > 180.0 {
            return Err(DomainError::InvalidLongitude(lng));
        }

        Ok(Self(lat, lng))
    }

    pub fn latitude(&self) -> f64 {
        self.0
    }

    pub fn longitude(&self) -> f64 {
        self.1
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
