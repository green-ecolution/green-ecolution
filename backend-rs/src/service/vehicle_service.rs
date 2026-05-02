use std::sync::Arc;

use crate::domain::{
    Id,
    shared::{
        pagination::{Page, Pagination},
        provenance::Provenance,
        water_capacity::WaterCapacity,
    },
    vehicle::{
        DrivingLicense, NumberPlate, Vehicle, VehicleDimension, VehicleDraft, VehicleModel,
        VehicleReader, VehicleSearchQuery, VehicleStatus, VehicleType, VehicleView, VehicleWriter,
    },
};

use super::ServiceError;

pub struct VehicleService {
    reader: Arc<dyn VehicleReader>,
    writer: Arc<dyn VehicleWriter>,
}

impl VehicleService {
    pub fn new(reader: Arc<dyn VehicleReader>, writer: Arc<dyn VehicleWriter>) -> Self {
        Self { reader, writer }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: VehicleSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<VehicleView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_by_type(
        &self,
        vehicle_type: VehicleType,
        pagination: Pagination,
    ) -> Result<Page<VehicleView>, ServiceError> {
        Ok(self.reader.view_by_type(vehicle_type, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(vehicle.id = %id))]
    pub async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, ServiceError> {
        Ok(self.reader.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(vehicle.id = %id))]
    pub async fn view_by_id(&self, id: Id<Vehicle>) -> Result<VehicleView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<VehicleView>, ServiceError> {
        Ok(self.reader.view_by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plate = %plate))]
    pub async fn by_plate(&self, plate: &NumberPlate) -> Result<Option<Vehicle>, ServiceError> {
        Ok(self.reader.by_plate(plate).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: VehicleDraft) -> Result<Vehicle, ServiceError> {
        Ok(self.writer.save_new(draft).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(vehicle.id = %id))]
    #[allow(clippy::too_many_arguments)]
    pub async fn replace(
        &self,
        id: Id<Vehicle>,
        number_plate: NumberPlate,
        description: Option<String>,
        water_capacity: WaterCapacity,
        status: VehicleStatus,
        vehicle_type: VehicleType,
        model: VehicleModel,
        driving_license: DrivingLicense,
        dimension: VehicleDimension,
        provenance: Provenance,
    ) -> Result<Vehicle, ServiceError> {
        let mut vehicle = self.reader.by_id(id).await?;
        vehicle.replace_details(
            number_plate,
            description,
            water_capacity,
            status,
            vehicle_type,
            model,
            driving_license,
            dimension,
            provenance,
        );
        self.writer.save(&vehicle).await?;
        Ok(vehicle)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(vehicle.id = %id))]
    pub async fn archive(&self, id: Id<Vehicle>) -> Result<(), ServiceError> {
        let mut vehicle = self.reader.by_id(id).await?;
        vehicle.archive(chrono::Utc::now());
        self.writer.save(&vehicle).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(vehicle.id = %id))]
    pub async fn delete(&self, id: Id<Vehicle>) -> Result<(), ServiceError> {
        Ok(self.writer.delete(id).await?)
    }
}
