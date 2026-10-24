use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use domain::{
    Id,
    capacity::CapacityKind,
    organization::Organization,
    shared::coordinates::Coordinate,
    start_point::{StartPoint, StartPointDraft, StartPointName, StartPointUpdate},
};

use super::vehicle::CapacityKindDto;
use crate::service::ServiceError;

/// Named departure/return point for routes. `replenishes` lists the capacity
/// kinds a vehicle restores here.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct StartPointResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub is_default: bool,
    pub replenishes: Vec<CapacityKindDto>,
    /// Organization this start point belongs to.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: String,
}

impl From<&StartPoint> for StartPointResponse {
    fn from(value: &StartPoint) -> Self {
        Self {
            id: value.id.value(),
            name: value.name.as_str().to_string(),
            lat: value.coordinate.latitude(),
            lon: value.coordinate.longitude(),
            is_default: value.is_default(),
            replenishes: value.replenishes().iter().map(|k| (*k).into()).collect(),
            organization_id: value.organization_id().to_string(),
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
    #[schema(example = json!(["water"]))]
    pub replenishes: Vec<CapacityKindDto>,
    /// Organization this start point belongs to. Defaults to the acting
    /// user's own organization (or the root organization in the demo bypass).
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub organization_id: Option<uuid::Uuid>,
}

impl StartPointRequest {
    fn coordinate(&self) -> Result<Coordinate, ServiceError> {
        Coordinate::new(self.lat, self.lon).map_err(ServiceError::from)
    }

    fn name(&self) -> Result<StartPointName, ServiceError> {
        StartPointName::new(self.name.clone()).map_err(ServiceError::from)
    }

    fn replenishes(&self) -> BTreeSet<CapacityKind> {
        self.replenishes.iter().map(|k| (*k).into()).collect()
    }

    pub fn into_draft(
        self,
        organization_id: Id<Organization>,
    ) -> Result<StartPointDraft, ServiceError> {
        Ok(StartPointDraft {
            name: self.name()?,
            coordinate: self.coordinate()?,
            replenishes: self.replenishes(),
            organization_id,
        })
    }

    pub fn into_update(self) -> Result<StartPointUpdate, ServiceError> {
        Ok(StartPointUpdate {
            name: self.name()?,
            coordinate: self.coordinate()?,
            replenishes: self.replenishes(),
        })
    }
}
