pub mod cluster;
pub mod evaluation;
pub mod tree;

use irrigation_domain::{
    cluster::SoilCondition as DomainSoilCondition,
    watering_status::WateringStatus as DomainWateringStatus,
};

/// Current watering status of a tree or tree cluster.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[schema(example = "good")]
pub enum WateringStatus {
    /// Soil moisture is sufficient — no watering needed.
    Good,
    /// Soil moisture is below optimal — watering recommended soon.
    Moderate,
    /// Soil moisture is critically low — immediate watering required.
    Bad,
    /// Recently watered — status will update after next sensor reading.
    #[serde(rename = "just_watered", alias = "just watered")]
    JustWatered,
    /// Status could not be determined (e.g. no sensor data available).
    Unknown,
}

/// KA5 soil texture class of a tree cluster site (KA5 short code).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[schema(example = "Lu")]
#[allow(non_camel_case_types)]
pub enum SoilCondition {
    Ss,
    Sl2,
    Sl3,
    Sl4,
    Slu,
    St2,
    St3,
    Su2,
    Su3,
    Su4,
    Ls2,
    Ls3,
    Ls4,
    Lt2,
    Lt3,
    Lts,
    Lu,
    Uu,
    Uls,
    Us,
    Ut2,
    Ut3,
    Ut4,
    Tt,
    Tl,
    Tu2,
    Tu3,
    Tu4,
    Ts2,
    Ts3,
    Ts4,
    #[serde(rename = "fS")]
    Fs,
    #[serde(rename = "mS")]
    Ms,
    #[serde(rename = "gS")]
    Gs,
    #[serde(rename = "unknown")]
    Unknown,
}

impl From<SoilCondition> for DomainSoilCondition {
    fn from(value: SoilCondition) -> Self {
        match value {
            SoilCondition::Ss => Self::Ss,
            SoilCondition::Sl2 => Self::Sl2,
            SoilCondition::Sl3 => Self::Sl3,
            SoilCondition::Sl4 => Self::Sl4,
            SoilCondition::Slu => Self::Slu,
            SoilCondition::St2 => Self::St2,
            SoilCondition::St3 => Self::St3,
            SoilCondition::Su2 => Self::Su2,
            SoilCondition::Su3 => Self::Su3,
            SoilCondition::Su4 => Self::Su4,
            SoilCondition::Ls2 => Self::Ls2,
            SoilCondition::Ls3 => Self::Ls3,
            SoilCondition::Ls4 => Self::Ls4,
            SoilCondition::Lt2 => Self::Lt2,
            SoilCondition::Lt3 => Self::Lt3,
            SoilCondition::Lts => Self::Lts,
            SoilCondition::Lu => Self::Lu,
            SoilCondition::Uu => Self::Uu,
            SoilCondition::Uls => Self::Uls,
            SoilCondition::Us => Self::Us,
            SoilCondition::Ut2 => Self::Ut2,
            SoilCondition::Ut3 => Self::Ut3,
            SoilCondition::Ut4 => Self::Ut4,
            SoilCondition::Tt => Self::Tt,
            SoilCondition::Tl => Self::Tl,
            SoilCondition::Tu2 => Self::Tu2,
            SoilCondition::Tu3 => Self::Tu3,
            SoilCondition::Tu4 => Self::Tu4,
            SoilCondition::Ts2 => Self::Ts2,
            SoilCondition::Ts3 => Self::Ts3,
            SoilCondition::Ts4 => Self::Ts4,
            SoilCondition::Fs => Self::Fs,
            SoilCondition::Ms => Self::Ms,
            SoilCondition::Gs => Self::Gs,
            SoilCondition::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainWateringStatus> for WateringStatus {
    fn from(value: DomainWateringStatus) -> Self {
        match value {
            DomainWateringStatus::Good => Self::Good,
            DomainWateringStatus::Moderate => Self::Moderate,
            DomainWateringStatus::Bad => Self::Bad,
            DomainWateringStatus::JustWatered => Self::JustWatered,
            DomainWateringStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<WateringStatus> for DomainWateringStatus {
    fn from(value: WateringStatus) -> Self {
        match value {
            WateringStatus::Good => Self::Good,
            WateringStatus::Moderate => Self::Moderate,
            WateringStatus::Bad => Self::Bad,
            WateringStatus::JustWatered => Self::JustWatered,
            WateringStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<DomainSoilCondition> for SoilCondition {
    fn from(value: DomainSoilCondition) -> Self {
        match value {
            DomainSoilCondition::Ss => Self::Ss,
            DomainSoilCondition::Sl2 => Self::Sl2,
            DomainSoilCondition::Sl3 => Self::Sl3,
            DomainSoilCondition::Sl4 => Self::Sl4,
            DomainSoilCondition::Slu => Self::Slu,
            DomainSoilCondition::St2 => Self::St2,
            DomainSoilCondition::St3 => Self::St3,
            DomainSoilCondition::Su2 => Self::Su2,
            DomainSoilCondition::Su3 => Self::Su3,
            DomainSoilCondition::Su4 => Self::Su4,
            DomainSoilCondition::Ls2 => Self::Ls2,
            DomainSoilCondition::Ls3 => Self::Ls3,
            DomainSoilCondition::Ls4 => Self::Ls4,
            DomainSoilCondition::Lt2 => Self::Lt2,
            DomainSoilCondition::Lt3 => Self::Lt3,
            DomainSoilCondition::Lts => Self::Lts,
            DomainSoilCondition::Lu => Self::Lu,
            DomainSoilCondition::Uu => Self::Uu,
            DomainSoilCondition::Uls => Self::Uls,
            DomainSoilCondition::Us => Self::Us,
            DomainSoilCondition::Ut2 => Self::Ut2,
            DomainSoilCondition::Ut3 => Self::Ut3,
            DomainSoilCondition::Ut4 => Self::Ut4,
            DomainSoilCondition::Tt => Self::Tt,
            DomainSoilCondition::Tl => Self::Tl,
            DomainSoilCondition::Tu2 => Self::Tu2,
            DomainSoilCondition::Tu3 => Self::Tu3,
            DomainSoilCondition::Tu4 => Self::Tu4,
            DomainSoilCondition::Ts2 => Self::Ts2,
            DomainSoilCondition::Ts3 => Self::Ts3,
            DomainSoilCondition::Ts4 => Self::Ts4,
            DomainSoilCondition::Fs => Self::Fs,
            DomainSoilCondition::Ms => Self::Ms,
            DomainSoilCondition::Gs => Self::Gs,
            DomainSoilCondition::Unknown => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod wire_format_tests {
    use super::*;

    /// A value with a space cannot serve as an i18n key segment.
    #[test]
    fn the_watering_status_contains_no_space() {
        let value = serde_json::to_string(&WateringStatus::JustWatered).unwrap();
        assert!(
            !value.contains(' '),
            "a wire value must be usable as a key segment, got {value}"
        );
    }

    #[test]
    fn the_watering_status_serializes_in_snake_case() {
        assert_eq!(
            serde_json::to_string(&WateringStatus::JustWatered).unwrap(),
            "\"just_watered\""
        );
    }

    /// The rename would otherwise be a breaking change. Accepting the old
    /// spelling on input keeps every existing caller working.
    #[test]
    fn the_previous_watering_status_spelling_is_still_accepted_on_input() {
        assert_eq!(
            serde_json::from_str::<WateringStatus>("\"just watered\"").unwrap(),
            WateringStatus::JustWatered
        );
    }

    #[test]
    fn the_canonical_watering_status_spelling_is_accepted_on_input() {
        assert_eq!(
            serde_json::from_str::<WateringStatus>("\"just_watered\"").unwrap(),
            WateringStatus::JustWatered
        );
    }
}
