//! Capacity a vehicle brings to a route and a station restores. Irrigation
//! counts water on board; waste collection counts free cargo space. Both are
//! liters, so one optimizer serves both.

use std::{fmt, ops::Add, str::FromStr};

use crate::{Id, shared::error::ValidationError, vehicle::Vehicle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CapacityKind {
    Water,
    CargoVolume,
}

impl CapacityKind {
    pub const ALL: [Self; 2] = [Self::Water, Self::CargoVolume];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Water => "water",
            Self::CargoVolume => "cargo_volume",
        }
    }
}

impl FromStr for CapacityKind {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "capacity.kind",
                reason: format!("unknown capacity kind '{s}'"),
            })
    }
}

impl fmt::Display for CapacityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Non-negative volume in liters.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Liters(f64);

impl Liters {
    pub fn new(liters: f64) -> Result<Self, ValidationError> {
        if liters.is_sign_negative() || liters.is_nan() {
            return Err(ValidationError::OutOfRange {
                field: "capacity.liters",
                min: 0.0,
                max: f64::INFINITY,
                got: liters,
            });
        }
        Ok(Self(liters))
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Add for Liters {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl fmt::Display for Liters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}L", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("vehicle {vehicle} has no {kind} capacity")]
pub struct CapacityMissing {
    pub vehicle: Id<Vehicle>,
    pub kind: CapacityKind,
}

/// Capacity a route can use: the transporter's own share when it is used,
/// plus the trailer. Every vehicle whose capacity is needed must declare the
/// kind, otherwise it does not fit the plan.
pub fn route_capacity(
    kind: CapacityKind,
    transporter: &Vehicle,
    trailer: Option<&Vehicle>,
    transporter_capacity_used: bool,
) -> Result<Liters, CapacityMissing> {
    let of = |v: &Vehicle| {
        v.capacity(kind).ok_or(CapacityMissing {
            vehicle: v.id,
            kind,
        })
    };
    let own = if transporter_capacity_used {
        of(transporter)?
    } else {
        Liters::zero()
    };
    let towed = match trailer {
        Some(t) => of(t)?,
        None => Liters::zero(),
    };
    Ok(own + towed)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use claims::{assert_err, assert_ok};

    use super::*;
    use crate::vehicle::{DrivingLicense, VehicleAvailability, VehicleSnapshot, VehicleType};

    fn vehicle_with(capacities: &[(CapacityKind, f64)]) -> Vehicle {
        Vehicle::reconstitute(VehicleSnapshot {
            id: uuid::Uuid::now_v7(),
            archived_at: None,
            number_plate: "FL GE 1234".to_string(),
            description: None,
            capacities: BTreeMap::from_iter(capacities.iter().copied()),
            availability: VehicleAvailability::Available,
            vehicle_type: VehicleType::Transporter,
            model: "Unimog".to_string(),
            driving_license: DrivingLicense::BE,
            height: 2.1,
            width: 2.0,
            length: 5.0,
            weight: 3500.0,
            provider: None,
            additional_info: None,
            organization_id: uuid::Uuid::now_v7(),
        })
    }

    #[derive(Debug, Clone)]
    struct ValidLitersFixture(f64);

    impl quickcheck::Arbitrary for ValidLitersFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self((u32::arbitrary(g) as f64) / 100.0)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn non_negative_liters_are_accepted(valid: ValidLitersFixture) -> bool {
        Liters::new(valid.0).is_ok()
    }

    #[test]
    fn zero_is_valid() {
        assert_ok!(Liters::new(0.0));
    }

    #[test]
    fn negative_value_is_rejected() {
        assert_err!(Liters::new(-0.1));
    }

    #[test]
    fn nan_is_rejected() {
        assert_err!(Liters::new(f64::NAN));
    }

    #[test]
    fn add_two_volumes() {
        let a = Liters::new(1000.0).unwrap();
        let b = Liters::new(500.0).unwrap();
        assert_eq!((a + b).value(), 1500.0);
    }

    #[test]
    fn display_format() {
        assert_eq!(Liters::new(2500.0).unwrap().to_string(), "2500L");
    }

    #[test]
    fn kinds_round_trip_through_their_string_form() {
        for kind in CapacityKind::ALL {
            assert_eq!(kind.as_str().parse::<CapacityKind>().unwrap(), kind);
        }
        assert_err!("sand".parse::<CapacityKind>());
    }

    #[test]
    fn route_capacity_adds_trailer_to_transporter() {
        let transporter = vehicle_with(&[(CapacityKind::Water, 1000.0)]);
        let trailer = vehicle_with(&[(CapacityKind::Water, 500.0)]);
        let c = route_capacity(CapacityKind::Water, &transporter, Some(&trailer), true).unwrap();
        assert_eq!(c.value(), 1500.0);
    }

    #[test]
    fn route_capacity_skips_unused_transporter_tank() {
        let transporter = vehicle_with(&[(CapacityKind::Water, 1000.0)]);
        let trailer = vehicle_with(&[(CapacityKind::Water, 500.0)]);
        let c = route_capacity(CapacityKind::Water, &transporter, Some(&trailer), false).unwrap();
        assert_eq!(c.value(), 500.0);
    }

    #[test]
    fn route_capacity_without_trailer_is_transporter_tank() {
        let transporter = vehicle_with(&[(CapacityKind::Water, 1000.0)]);
        let c = route_capacity(CapacityKind::Water, &transporter, None, true).unwrap();
        assert_eq!(c.value(), 1000.0);
    }

    #[test]
    fn a_transporter_without_the_kind_does_not_fit_when_used() {
        let transporter = vehicle_with(&[(CapacityKind::CargoVolume, 1000.0)]);
        let err = route_capacity(CapacityKind::Water, &transporter, None, true).unwrap_err();
        assert_eq!(err.kind, CapacityKind::Water);
    }

    #[test]
    fn an_unused_transporter_may_lack_the_kind() {
        let transporter = vehicle_with(&[]);
        let trailer = vehicle_with(&[(CapacityKind::Water, 2000.0)]);
        let c = route_capacity(CapacityKind::Water, &transporter, Some(&trailer), false).unwrap();
        assert_eq!(c.value(), 2000.0);
    }

    #[test]
    fn a_trailer_without_the_kind_does_not_fit() {
        let transporter = vehicle_with(&[(CapacityKind::Water, 500.0)]);
        let trailer = vehicle_with(&[(CapacityKind::CargoVolume, 2000.0)]);
        assert!(route_capacity(CapacityKind::Water, &transporter, Some(&trailer), true).is_err());
    }

    #[test]
    fn cargo_volume_is_counted_like_water() {
        let transporter = vehicle_with(&[(CapacityKind::CargoVolume, 800.0)]);
        let trailer = vehicle_with(&[(CapacityKind::CargoVolume, 1200.0)]);
        let c = route_capacity(
            CapacityKind::CargoVolume,
            &transporter,
            Some(&trailer),
            true,
        )
        .unwrap();
        assert_eq!(c.value(), 2000.0);
    }
}
