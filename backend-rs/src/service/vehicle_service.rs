use std::sync::Arc;

use crate::domain::{
    Id,
    shared::pagination::{Page, Pagination},
    vehicle::{
        Vehicle, VehicleCreate, VehicleQuery, VehicleRepository, VehicleUpdate,
    },
};

use super::ServiceError;

pub struct VehicleService {
    vehicle_repo: Arc<dyn VehicleRepository>,
}

impl VehicleService {
    pub fn new(vehicle_repo: Arc<dyn VehicleRepository>) -> Self {
        Self { vehicle_repo }
    }

    pub async fn all(
        &self,
        query: VehicleQuery,
        pagination: Pagination,
    ) -> Result<Page<Vehicle>, ServiceError> {
        Ok(self.vehicle_repo.all(query, pagination).await?)
    }

    pub async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, ServiceError> {
        Ok(self.vehicle_repo.by_id(id).await?)
    }

    pub async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, ServiceError> {
        Ok(self.vehicle_repo.by_ids(ids).await?)
    }

    pub async fn by_plate(&self, plate: &str) -> Result<Vehicle, ServiceError> {
        Ok(self.vehicle_repo.by_plate(plate).await?)
    }

    pub async fn create(&self, input: VehicleCreate) -> Result<Vehicle, ServiceError> {
        Ok(self.vehicle_repo.create(input).await?)
    }

    pub async fn update(
        &self,
        id: Id<Vehicle>,
        input: VehicleUpdate,
    ) -> Result<Vehicle, ServiceError> {
        Ok(self.vehicle_repo.update(id, input).await?)
    }

    pub async fn archive(&self, id: Id<Vehicle>) -> Result<(), ServiceError> {
        Ok(self.vehicle_repo.archive(id).await?)
    }

    pub async fn delete(&self, id: Id<Vehicle>) -> Result<(), ServiceError> {
        Ok(self.vehicle_repo.delete(id).await?)
    }
}
