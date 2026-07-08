use serde::Serialize;

/// Named departure/return point for watering routes. The first entry is the default.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct StartPointResponse {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}
