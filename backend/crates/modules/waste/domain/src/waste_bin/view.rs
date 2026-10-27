use chrono::{DateTime, Utc};

use domain::{
    Id,
    organization::Organization,
    sensor::SensorId,
    shared::{coordinates::Coordinate, provenance::Provenance},
};

use crate::fill::FillMeasurement;
use crate::values::{BinDepth, BinVolume, WasteBinNumber};
use crate::waste_bin::WasteBin;

/// Read model for a waste bin; `created_at` is derived from the v7 id.
#[derive(Debug, Clone)]
pub struct WasteBinView {
    pub id: Id<WasteBin>,
    pub number: WasteBinNumber,
    pub coordinate: Coordinate,
    pub volume: BinVolume,
    pub depth: BinDepth,
    pub organization_id: Id<Organization>,
    pub provenance: Provenance,
    pub fill: Option<FillMeasurement>,
    pub last_emptied_at: Option<DateTime<Utc>>,
    pub sensor_id: Option<SensorId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
