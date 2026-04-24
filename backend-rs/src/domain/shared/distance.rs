use std::{fmt::Display, ops::Add};

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance(f64);

impl Distance {
    pub fn new(meters: f64) -> Result<Self, DomainError> {
        if meters.is_sign_negative() {
            return Err(DomainError::InvalidDistance(meters));
        }

        Ok(Distance(meters))
    }

    pub fn meters(&self) -> f64 {
        self.0
    }
}

impl Add for Distance {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.meters() + rhs.meters())
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}m", self.meters())
    }
}
