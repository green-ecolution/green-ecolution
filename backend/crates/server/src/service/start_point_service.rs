use std::sync::Arc;

use domain::{
    Id,
    start_point::{
        StartPoint, StartPointDraft, StartPointReader, StartPointUpdate, StartPointWriter,
    },
};

use super::ServiceError;

pub struct StartPointService {
    reader: Arc<dyn StartPointReader>,
    writer: Arc<dyn StartPointWriter>,
}

impl StartPointService {
    pub fn new(reader: Arc<dyn StartPointReader>, writer: Arc<dyn StartPointWriter>) -> Self {
        Self { reader, writer }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self) -> Result<Vec<StartPoint>, ServiceError> {
        Ok(self.reader.all().await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(start_point.id = %id))]
    pub async fn by_id(&self, id: Id<StartPoint>) -> Result<StartPoint, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: StartPointDraft) -> Result<StartPoint, ServiceError> {
        Ok(self.writer.save_new(draft).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(start_point.id = %id))]
    pub async fn update(
        &self,
        id: Id<StartPoint>,
        update: StartPointUpdate,
    ) -> Result<StartPoint, ServiceError> {
        let mut sp = self.reader.by_id(id).await?;
        sp.rename(update.name);
        sp.relocate(update.coordinate);
        sp.set_watering_point(update.watering_point);
        self.writer.save(&sp).await?;
        Ok(sp)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(start_point.id = %id))]
    pub async fn set_default(&self, id: Id<StartPoint>) -> Result<(), ServiceError> {
        // by_id first so an unknown id maps to 404, not a silent no-op.
        self.reader.by_id(id).await?;
        Ok(self.writer.set_default(id).await?)
    }

    /// Deleting the default start point is rejected: there must always be
    /// exactly one default while any start point exists.
    #[tracing::instrument(level = "debug", skip_all, fields(start_point.id = %id))]
    pub async fn delete(&self, id: Id<StartPoint>) -> Result<(), ServiceError> {
        let sp = self.reader.by_id(id).await?;
        if sp.is_default() {
            return Err(ServiceError::InvalidInput(
                "cannot delete the default start point; set another default first".into(),
            ));
        }
        Ok(self.writer.delete(id).await?)
    }
}
