/// Soil-moisture assessment derived from sensor readings.
///
/// Used on both `Tree` (updated when a sensor reading arrives or a sensor is
/// detached) and `TreeCluster` (derived as the majority status across member
/// trees). `Unknown` is the default when no sensor is attached or no readings
/// have been processed yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "watering_status", rename_all = "snake_case")
)]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    #[cfg_attr(feature = "sqlx", sqlx(rename = "just watered"))]
    JustWatered,
    Unknown,
}
