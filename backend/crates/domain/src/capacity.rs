//! Capacity a vehicle brings to a route and a station restores. The kinds are
//! declared by modules; the core only knows them as keys. An amount carries no
//! unit: the core only ever compares and adds amounts of one kind, and the
//! unit a kind's module declares serves display.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    ops::Add,
    str::FromStr,
};

use crate::{
    Id,
    module::ModuleKey,
    shared::{error::ValidationError, key::validate_key},
    vehicle::Vehicle,
};

/// Owned rather than `&'static str`: a stored kind no registered module
/// declares must still be read back and returned unchanged.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapacityKind(String);

impl CapacityKind {
    const FIELD: &'static str = "capacity.kind";

    pub fn new(value: &str) -> Result<Self, ValidationError> {
        validate_key(Self::FIELD, value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for CapacityKind {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for CapacityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a module declares about one of its capacity kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapacityKindSpec {
    pub kind: CapacityKind,
    pub unit: &'static str,
    /// A tour of this kind must end replenished, e.g. unload before it returns
    /// to the depot.
    pub returns_replenished: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown capacity kind '{0}'")]
pub struct UnknownCapacityKind(pub CapacityKind);

/// Every capacity kind the registered modules declare, each with its owner.
#[derive(Debug, Clone, Default)]
pub struct CapacityCatalog {
    kinds: BTreeMap<CapacityKind, (ModuleKey, CapacityKindSpec)>,
}

impl CapacityCatalog {
    pub fn new(
        declared: impl IntoIterator<Item = (ModuleKey, CapacityKindSpec)>,
    ) -> Result<Self, ValidationError> {
        let mut kinds = BTreeMap::new();
        for (module, spec) in declared {
            if spec.unit.is_empty() {
                return Err(ValidationError::EmptyString {
                    field: "capacity.unit",
                });
            }
            if kinds.contains_key(&spec.kind) {
                return Err(ValidationError::InvalidFormat {
                    field: CapacityKind::FIELD,
                    reason: format!("capacity kind '{}' is declared twice", spec.kind),
                });
            }
            kinds.insert(spec.kind.clone(), (module, spec));
        }
        Ok(Self { kinds })
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn spec(&self, kind: &CapacityKind) -> Option<&CapacityKindSpec> {
        self.kinds.get(kind).map(|(_, spec)| spec)
    }

    pub fn owner(&self, kind: &CapacityKind) -> Option<&ModuleKey> {
        self.kinds.get(kind).map(|(module, _)| module)
    }

    /// A kind no module declares may stay where it is already stored, so
    /// removing a module never makes its vehicles or stations unsavable, but it
    /// cannot be added anew.
    pub fn admit<'a>(
        &self,
        requested: impl IntoIterator<Item = &'a CapacityKind>,
        stored: &BTreeSet<CapacityKind>,
    ) -> Result<(), UnknownCapacityKind> {
        match requested
            .into_iter()
            .find(|kind| !self.kinds.contains_key(*kind) && !stored.contains(*kind))
        {
            Some(kind) => Err(UnknownCapacityKind(kind.clone())),
            None => Ok(()),
        }
    }
}

/// Non-negative amount of one capacity kind.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Amount(f64);

impl Amount {
    pub fn new(amount: f64) -> Result<Self, ValidationError> {
        if amount.is_sign_negative() || amount.is_nan() {
            return Err(ValidationError::OutOfRange {
                field: "capacity.amount",
                min: 0.0,
                max: f64::INFINITY,
                got: amount,
            });
        }
        Ok(Self(amount))
    }

    pub fn zero() -> Self {
        Self(0.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("vehicle {vehicle} has no {kind} capacity")]
pub struct CapacityMissing {
    pub vehicle: Id<Vehicle>,
    pub kind: CapacityKind,
}

/// Capacity a route can use: the transporter's own share when it is used,
/// plus the trailer. Every vehicle whose capacity is needed must declare the
/// kind, otherwise it does not fit the plan.
pub fn route_capacity(
    kind: &CapacityKind,
    transporter: &Vehicle,
    trailer: Option<&Vehicle>,
    transporter_capacity_used: bool,
) -> Result<Amount, CapacityMissing> {
    let of = |v: &Vehicle| {
        v.capacity(kind).ok_or_else(|| CapacityMissing {
            vehicle: v.id,
            kind: kind.clone(),
        })
    };
    let own = if transporter_capacity_used {
        of(transporter)?
    } else {
        Amount::zero()
    };
    let towed = match trailer {
        Some(t) => of(t)?,
        None => Amount::zero(),
    };
    Ok(own + towed)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use claims::{assert_err, assert_ok};

    use super::*;
    use crate::vehicle::{DrivingLicense, VehicleAvailability, VehicleSnapshot, VehicleType};

    fn kind(key: &str) -> CapacityKind {
        CapacityKind::new(key).unwrap()
    }

    fn module(key: &str) -> ModuleKey {
        ModuleKey::new(key).unwrap()
    }

    fn spec(key: &str, returns_replenished: bool) -> CapacityKindSpec {
        CapacityKindSpec {
            kind: kind(key),
            unit: "l",
            returns_replenished,
        }
    }

    fn vehicle_with(capacities: &[(&str, f64)]) -> Vehicle {
        Vehicle::reconstitute(VehicleSnapshot {
            id: uuid::Uuid::now_v7(),
            archived_at: None,
            number_plate: "FL GE 1234".to_string(),
            description: None,
            capacities: capacities
                .iter()
                .map(|(k, v)| (kind(k), *v))
                .collect::<BTreeMap<_, _>>(),
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
    struct ValidAmountFixture(f64);

    impl quickcheck::Arbitrary for ValidAmountFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self((u32::arbitrary(g) as f64) / 100.0)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn non_negative_amounts_are_accepted(valid: ValidAmountFixture) -> bool {
        Amount::new(valid.0).is_ok()
    }

    #[test]
    fn zero_is_valid() {
        assert_ok!(Amount::new(0.0));
    }

    #[test]
    fn negative_value_is_rejected_as_capacity_amount() {
        assert!(matches!(
            Amount::new(-0.1),
            Err(ValidationError::OutOfRange {
                field: "capacity.amount",
                ..
            })
        ));
    }

    #[test]
    fn nan_is_rejected() {
        assert_err!(Amount::new(f64::NAN));
    }

    #[test]
    fn add_two_amounts() {
        let a = Amount::new(1000.0).unwrap();
        let b = Amount::new(500.0).unwrap();
        assert_eq!((a + b).value(), 1500.0);
    }

    #[test]
    fn display_carries_no_unit() {
        assert_eq!(Amount::new(2500.0).unwrap().to_string(), "2500");
    }

    #[test]
    fn a_kind_follows_the_stable_key_format() {
        assert_eq!(kind("water").as_str(), "water");
        assert_eq!(kind("cargo_volume").to_string(), "cargo_volume");
        assert_eq!("sand".parse::<CapacityKind>().unwrap(), kind("sand"));
        for invalid in ["", "Water", "a-b", "Sand", &"a".repeat(33)] {
            assert_err!(CapacityKind::new(invalid), "{invalid:?} must be refused");
        }
    }

    #[test]
    fn the_catalog_rejects_a_kind_declared_twice() {
        let err = CapacityCatalog::new([
            (module("irrigation"), spec("water", false)),
            (module("waste"), spec("water", true)),
        ])
        .unwrap_err();
        assert!(err.to_string().contains("water"), "{err}");
    }

    #[test]
    fn the_catalog_rejects_an_empty_unit() {
        let mut no_unit = spec("water", false);
        no_unit.unit = "";
        assert_err!(CapacityCatalog::new([(module("irrigation"), no_unit)]));
    }

    #[test]
    fn the_catalog_names_owner_and_spec() {
        let catalog = CapacityCatalog::new([
            (module("irrigation"), spec("water", false)),
            (module("waste"), spec("cargo_volume", true)),
        ])
        .unwrap();
        assert_eq!(catalog.owner(&kind("water")), Some(&module("irrigation")));
        assert!(
            catalog
                .spec(&kind("cargo_volume"))
                .unwrap()
                .returns_replenished
        );
        assert!(catalog.spec(&kind("sand")).is_none());
        assert!(CapacityCatalog::empty().spec(&kind("water")).is_none());
    }

    #[test]
    fn admit_accepts_declared_and_already_stored_kinds() {
        let catalog = CapacityCatalog::new([(module("irrigation"), spec("water", false))]).unwrap();
        let stored = BTreeSet::from([kind("sand")]);
        assert_ok!(catalog.admit([&kind("water"), &kind("sand")], &stored));
    }

    #[test]
    fn admit_refuses_a_new_undeclared_kind() {
        let catalog = CapacityCatalog::new([(module("irrigation"), spec("water", false))]).unwrap();
        let err = catalog
            .admit([&kind("water"), &kind("sand")], &BTreeSet::new())
            .unwrap_err();
        assert_eq!(err, UnknownCapacityKind(kind("sand")));
    }

    #[test]
    fn route_capacity_adds_trailer_to_transporter() {
        let transporter = vehicle_with(&[("water", 1000.0)]);
        let trailer = vehicle_with(&[("water", 500.0)]);
        let c = route_capacity(&kind("water"), &transporter, Some(&trailer), true).unwrap();
        assert_eq!(c.value(), 1500.0);
    }

    #[test]
    fn route_capacity_skips_unused_transporter_tank() {
        let transporter = vehicle_with(&[("water", 1000.0)]);
        let trailer = vehicle_with(&[("water", 500.0)]);
        let c = route_capacity(&kind("water"), &transporter, Some(&trailer), false).unwrap();
        assert_eq!(c.value(), 500.0);
    }

    #[test]
    fn route_capacity_without_trailer_is_transporter_tank() {
        let transporter = vehicle_with(&[("water", 1000.0)]);
        let c = route_capacity(&kind("water"), &transporter, None, true).unwrap();
        assert_eq!(c.value(), 1000.0);
    }

    #[test]
    fn a_transporter_without_the_kind_does_not_fit_when_used() {
        let transporter = vehicle_with(&[("cargo_volume", 1000.0)]);
        let err = route_capacity(&kind("water"), &transporter, None, true).unwrap_err();
        assert_eq!(err.kind, kind("water"));
    }

    #[test]
    fn an_unused_transporter_may_lack_the_kind() {
        let transporter = vehicle_with(&[]);
        let trailer = vehicle_with(&[("water", 2000.0)]);
        let c = route_capacity(&kind("water"), &transporter, Some(&trailer), false).unwrap();
        assert_eq!(c.value(), 2000.0);
    }

    #[test]
    fn a_trailer_without_the_kind_does_not_fit() {
        let transporter = vehicle_with(&[("water", 500.0)]);
        let trailer = vehicle_with(&[("cargo_volume", 2000.0)]);
        assert!(route_capacity(&kind("water"), &transporter, Some(&trailer), true).is_err());
    }

    #[test]
    fn every_kind_is_counted_alike() {
        let transporter = vehicle_with(&[("cargo_volume", 800.0)]);
        let trailer = vehicle_with(&[("cargo_volume", 1200.0)]);
        let c = route_capacity(&kind("cargo_volume"), &transporter, Some(&trailer), true).unwrap();
        assert_eq!(c.value(), 2000.0);
    }
}
