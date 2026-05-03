/// Soil texture classification for a cluster's planting site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "tree_soil_condition", rename_all = "snake_case")
)]
pub enum SoilCondition {
    Schluffig,
    Sandig,
    Lehmig,
    Tonig,
    Unknown,
}
