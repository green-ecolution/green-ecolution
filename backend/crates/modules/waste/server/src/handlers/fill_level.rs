use std::sync::Arc;

use chrono::Utc;
use domain::{
    Id, RepositoryError,
    events::{DomainEvent, SensorReadings},
};
use server::service::event_bus::{EventHandler, EventHandlerError};
use waste_domain::waste_bin::{WasteBinReader, WasteBinWriter};

use crate::targets::is_bin;

/// Records a bin's fill level from the distance reading of the sensor attached to it.
pub struct FillLevelFromSensor {
    reader: Arc<dyn WasteBinReader>,
    writer: Arc<dyn WasteBinWriter>,
}

impl FillLevelFromSensor {
    pub fn new(reader: Arc<dyn WasteBinReader>, writer: Arc<dyn WasteBinWriter>) -> Self {
        Self { reader, writer }
    }
}

#[async_trait::async_trait]
impl EventHandler for FillLevelFromSensor {
    fn name(&self) -> &str {
        "waste_fill_level"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let DomainEvent::SensorDataReceived(payload) = event else {
            return Ok(vec![]);
        };
        let Some(target) = payload.target.as_ref().filter(|t| is_bin(t)) else {
            return Ok(vec![]);
        };
        let SensorReadings::Distances(readings) = &payload.readings else {
            return Ok(vec![]);
        };
        let Some(reading) = readings.first() else {
            return Ok(vec![]);
        };
        let mut bin = match self.reader.by_id(Id::new(target.id)).await {
            Ok(bin) => bin,
            Err(RepositoryError::NotFound) => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };
        // The payload carries no timestamp and the handler runs right after ingest.
        if bin.record_fill_level(reading.centimeters, Utc::now()) {
            self.writer.save(&bin).await?;
        }
        Ok(vec![])
    }
}
