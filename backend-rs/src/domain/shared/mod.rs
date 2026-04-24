pub mod coordinates;
pub mod distance;
pub mod provider_info;
pub mod water_capacity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    JustWatered,
    Unknown,
}
