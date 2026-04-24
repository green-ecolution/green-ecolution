use chrono::{DateTime, Utc};

use crate::domain::{Id, shared::{coordinates::Coordinate, provider_info::ProviderInfo}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    Online,
    Offline,
}

#[derive(Debug, Clone)]
pub struct Sensor {
    id: Id<Self>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: Option<SensorStatus>,
    latest_data: Option<SensorData>,
    coordinates: Coordinate,
    provider_info: ProviderInfo,
}

impl Sensor {
    pub fn new(
        id: Id<Self>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        status: Option<SensorStatus>,
        latest_data: Option<SensorData>,
        coordinates: Coordinate,
        provider_info: ProviderInfo,
    ) -> Self {
        Self { id, created_at, updated_at, status, latest_data, coordinates, provider_info }
    }

    pub fn id(&self) -> &Id<Self> { &self.id }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn status(&self) -> Option<SensorStatus> { self.status }
    pub fn latest_data(&self) -> Option<&SensorData> { self.latest_data.as_ref() }
    pub fn coordinates(&self) -> &Coordinate { &self.coordinates }
    pub fn provider_info(&self) -> &ProviderInfo { &self.provider_info }
}

#[derive(Debug, Clone)]
pub struct SensorData {
    id: Id<Self>,
    sensor_id: Id<Sensor>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    // data
}

impl SensorData {
    pub fn new(
        id: Id<Self>,
        sensor_id: Id<Sensor>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self { id, sensor_id, created_at, updated_at }
    }

    pub fn id(&self) -> &Id<Self> { &self.id }
    pub fn sensor_id(&self) -> &Id<Sensor> { &self.sensor_id }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

#[derive(Debug)]
pub struct SensorCreate {
    pub id: Id<Sensor>,
    pub status: SensorStatus,
    pub latest_data: Option<SensorData>,
    pub coordinate: Coordinate,
    pub provider_info: ProviderInfo,
}

#[derive(Debug, Default)]
pub struct SensorUpdate {
    pub status: Option<SensorStatus>,
    pub latest_data: Option<SensorData>,
    pub coordinate: Option<Coordinate>,
    pub provider_info: Option<ProviderInfo>,
}
