use std::{fmt::Display, ops::Add};

use crate::domain::DomainError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterCapacity(f64);

impl WaterCapacity {
    pub fn new(liters: f64) -> Result<Self, DomainError> {
        if liters.is_sign_negative() {
            return Err(DomainError::InvalidWaterCapacity(liters));
        }
        Ok(Self(liters))
    }

    pub fn liters(&self) -> f64 {
        self.0
    }
}

impl Add for WaterCapacity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Display for WaterCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}L", self.0)
    }
}
