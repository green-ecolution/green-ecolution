/// KA5 fine soil texture class (Bodenkundliche Kartieranleitung, 5. Aufl.) for a
/// cluster's planting site. Drives the volumetric watering-status calibration.
///
/// Serialized/stored as the exact KA5 short code (`Ss`, `Sl2`, `fS`, …); the
/// pure-sand fractions `fS`/`mS`/`gS` need explicit renames because Rust
/// identifiers cannot start lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(type_name = "tree_soil_condition"))]
#[allow(non_camel_case_types)] // KA5 codes (Sl2, fS, …) are the canonical identifiers
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
    #[cfg_attr(feature = "sqlx", sqlx(rename = "fS"))]
    #[serde(rename = "fS")]
    Fs,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "mS"))]
    #[serde(rename = "mS")]
    Ms,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "gS"))]
    #[serde(rename = "gS")]
    Gs,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "unknown"))]
    #[serde(rename = "unknown")]
    Unknown,
}
