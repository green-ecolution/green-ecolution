use async_trait::async_trait;

use domain::{
    Id, RepositoryError,
    authorization::Visibility,
    shared::pagination::{Pagination, SearchPage},
};

use crate::waste_bin::{WasteBin, WasteBinDraft, WasteBinSearchQuery, WasteBinView};

#[async_trait]
pub trait WasteBinReader: Send + Sync {
    async fn by_id(&self, id: Id<WasteBin>) -> Result<WasteBin, RepositoryError>;
    async fn by_ids(&self, ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError>;

    async fn view_by_id(&self, id: Id<WasteBin>) -> Result<WasteBinView, RepositoryError>;
    async fn view_by_ids(&self, ids: &[Id<WasteBin>])
    -> Result<Vec<WasteBinView>, RepositoryError>;
    async fn view_search(
        &self,
        query: WasteBinSearchQuery,
        pagination: Pagination,
        visibility: &Visibility,
    ) -> Result<SearchPage<WasteBinView>, RepositoryError>;
}

#[async_trait]
pub trait WasteBinWriter: Send + Sync {
    async fn save_new(&self, draft: WasteBinDraft) -> Result<WasteBin, RepositoryError>;
    async fn save(&self, bin: &WasteBin) -> Result<(), RepositoryError>;
    async fn delete(&self, id: Id<WasteBin>) -> Result<(), RepositoryError>;
}
