use serde::{Deserialize, Serialize};

use domain::{
    shared::coordinates::Coordinate,
    start_point::{StartPoint, StartPointDraft, StartPointName, StartPointUpdate},
};

use crate::service::ServiceError;

/// Named departure/return point for watering routes. `watering_point` marks it
/// as a water refill station.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct StartPointResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub is_default: bool,
    pub watering_point: bool,
}

impl From<&StartPoint> for StartPointResponse {
    fn from(value: &StartPoint) -> Self {
        Self {
            id: value.id.value(),
            name: value.name.as_str().to_string(),
            lat: value.coordinate.latitude(),
            lon: value.coordinate.longitude(),
            is_default: value.is_default(),
            watering_point: value.watering_point(),
        }
    }
}

/// Create/replace payload for a start point.
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct StartPointRequest {
    #[schema(example = "Betriebshof Schleswiger Straße")]
    pub name: String,
    #[schema(example = 54.76879146396569)]
    pub lat: f64,
    #[schema(example = 9.434803531218018)]
    pub lon: f64,
    #[serde(default)]
    pub watering_point: bool,
}

impl StartPointRequest {
    fn coordinate(&self) -> Result<Coordinate, ServiceError> {
        Coordinate::new(self.lat, self.lon).map_err(|e| ServiceError::InvalidInput(e.to_string()))
    }

    fn name(&self) -> Result<StartPointName, ServiceError> {
        StartPointName::new(self.name.clone())
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))
    }

    pub fn into_draft(self) -> Result<StartPointDraft, ServiceError> {
        Ok(StartPointDraft {
            name: self.name()?,
            coordinate: self.coordinate()?,
            watering_point: self.watering_point,
        })
    }

    pub fn into_update(self) -> Result<StartPointUpdate, ServiceError> {
        Ok(StartPointUpdate {
            name: self.name()?,
            coordinate: self.coordinate()?,
            watering_point: self.watering_point,
        })
    }
}
