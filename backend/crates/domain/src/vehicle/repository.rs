use async_trait::async_trait;

use crate::{
    Id, RepositoryError,
    authorization::Visibility,
    shared::pagination::{Page, Pagination},
    vehicle::{NumberPlate, Vehicle, VehicleDraft, VehicleSearchQuery, VehicleType, VehicleView},
};

/// Read-side access to vehicles, including aggregate hydration and the
/// HTTP-friendly [`VehicleView`] read model.
#[async_trait]
pub trait VehicleReader: Send + Sync {
    async fn by_id(&self, id: Id<Vehicle>) -> Result<Vehicle, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<Vehicle>, RepositoryError>;
    async fn by_plate(&self, plate: &NumberPlate) -> Result<Option<Vehicle>, RepositoryError>;

    async fn view_by_id(&self, id: Id<Vehicle>) -> Result<VehicleView, RepositoryError>;
    async fn view_by_ids(&self, ids: &[Id<Vehicle>]) -> Result<Vec<VehicleView>, RepositoryError>;
    async fn view_search(
        &self,
        query: VehicleSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<VehicleView>, RepositoryError>;
    async fn view_by_type(
        &self,
        vehicle_type: VehicleType,
        pagination: Pagination,
        visible: Visibility,
    ) -> Result<Page<VehicleView>, RepositoryError>;
}

/// Write-side access to vehicles.
#[async_trait]
pub trait VehicleWriter: Send + Sync {
    async fn save_new(&self, draft: VehicleDraft) -> Result<Vehicle, RepositoryError>;
    async fn save(&self, vehicle: &Vehicle) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<Vehicle>) -> Result<(), RepositoryError>;
}
