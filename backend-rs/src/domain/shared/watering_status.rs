#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "watering_status", rename_all = "snake_case")]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    #[sqlx(rename = "just watered")]
    JustWatered,
    Unknown,
}
