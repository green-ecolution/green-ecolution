use crate::shared::error::ValidationError;

crate::newtype_nonempty! {
    /// Street line including house number.
    Street, "address.street", 1, 200
}

crate::newtype_nonempty! {
    /// City or municipality name.
    City, "address.city", 1, 120
}

const POSTAL_CODE_MIN: usize = 4;
const POSTAL_CODE_MAX: usize = 10;

/// Digits-only postal code. German codes are five digits; the wider bound keeps
/// neighbouring countries usable without opening the field up to free text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostalCode(String);

impl PostalCode {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        const FIELD: &str = "address.postal_code";
        let trimmed = value.into().trim().to_string();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyString { field: FIELD });
        }
        if !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err(ValidationError::InvalidFormat {
                field: FIELD,
                reason: "must contain digits only".to_string(),
            });
        }
        let len = trimmed.len();
        if len < POSTAL_CODE_MIN {
            return Err(ValidationError::TooShort {
                field: FIELD,
                min: POSTAL_CODE_MIN,
                got: len,
            });
        }
        if len > POSTAL_CODE_MAX {
            return Err(ValidationError::TooLong {
                field: FIELD,
                max: POSTAL_CODE_MAX,
                got: len,
            });
        }
        Ok(Self(trimmed))
    }

    pub fn reconstitute(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PostalCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A postal address. The three parts only ever exist together — there is no
/// address that carries just a postal code.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    street: Street,
    postal_code: PostalCode,
    city: City,
}

impl Address {
    pub fn new(street: Street, postal_code: PostalCode, city: City) -> Self {
        Self {
            street,
            postal_code,
            city,
        }
    }

    #[doc(hidden)]
    pub fn reconstitute(
        street: Option<String>,
        postal_code: Option<String>,
        city: Option<String>,
    ) -> Option<Self> {
        match (street, postal_code, city) {
            (Some(street), Some(postal_code), Some(city)) => Some(Self {
                street: Street::reconstitute(street),
                postal_code: PostalCode::reconstitute(postal_code),
                city: City::reconstitute(city),
            }),
            (None, None, None) => None,
            _ => panic!(
                "address parts are all-or-nothing; a partial triple means the invariant was violated before rehydration"
            ),
        }
    }

    pub fn street(&self) -> &Street {
        &self.street
    }

    pub fn postal_code(&self) -> &PostalCode {
        &self.postal_code
    }

    pub fn city(&self) -> &City {
        &self.city
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn address(street: &str, postal: &str, city: &str) -> Address {
        Address::new(
            Street::new(street).unwrap(),
            PostalCode::new(postal).unwrap(),
            City::new(city).unwrap(),
        )
    }

    #[test]
    fn accepts_a_german_address() {
        let a = address("Nordergraben 12", "24937", "Flensburg");
        assert_eq!(a.street().as_str(), "Nordergraben 12");
        assert_eq!(a.postal_code().as_str(), "24937");
        assert_eq!(a.city().as_str(), "Flensburg");
    }

    #[test]
    fn postal_code_trims_surrounding_whitespace() {
        assert_eq!(PostalCode::new("  24937 ").unwrap().as_str(), "24937");
    }

    #[test]
    fn postal_code_rejects_letters() {
        let err = PostalCode::new("24A37").unwrap_err();
        assert!(matches!(
            err,
            ValidationError::InvalidFormat {
                field: "address.postal_code",
                ..
            }
        ));
    }

    #[test]
    fn postal_code_rejects_too_few_digits() {
        assert_err!(PostalCode::new("249"));
    }

    #[test]
    fn postal_code_rejects_too_many_digits() {
        assert_err!(PostalCode::new("12345678901"));
    }

    #[test]
    fn postal_code_rejects_empty() {
        let err = PostalCode::new("   ").unwrap_err();
        assert_eq!(
            err,
            ValidationError::EmptyString {
                field: "address.postal_code"
            }
        );
    }

    #[test]
    fn street_and_city_reject_empty() {
        assert_err!(Street::new(" "));
        assert_err!(City::new(""));
    }

    #[test]
    fn street_rejects_over_two_hundred_chars() {
        assert_err!(Street::new("a".repeat(201)));
        assert_ok!(Street::new("a".repeat(200)));
    }

    #[test]
    fn city_rejects_over_one_hundred_twenty_chars() {
        assert_err!(City::new("a".repeat(121)));
        assert_ok!(City::new("a".repeat(120)));
    }

    #[test]
    fn reconstitute_maps_a_full_triple() {
        let a = Address::reconstitute(
            Some("Nordergraben 12".into()),
            Some("24937".into()),
            Some("Flensburg".into()),
        );
        assert_eq!(a, Some(address("Nordergraben 12", "24937", "Flensburg")));
    }

    #[test]
    fn reconstitute_maps_all_none_to_none() {
        assert_eq!(Address::reconstitute(None, None, None), None);
    }

    #[test]
    #[should_panic(expected = "all-or-nothing")]
    fn reconstitute_panics_on_a_partial_triple() {
        Address::reconstitute(Some("Nordergraben 12".into()), None, None);
    }
}
