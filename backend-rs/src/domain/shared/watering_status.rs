#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WateringStatus {
    Good,
    Moderate,
    Bad,
    JustWatered,
    Unknown,
}
