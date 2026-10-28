pub mod emptied;
pub mod fill_level;

use std::sync::Arc;

use server::service::event_bus::EventHandler;
use waste_domain::waste_bin::{WasteBinReader, WasteBinWriter};

pub fn all(
    bins: Arc<dyn WasteBinReader>,
    bin_writer: Arc<dyn WasteBinWriter>,
) -> Vec<Arc<dyn EventHandler>> {
    vec![
        Arc::new(fill_level::FillLevelFromSensor::new(
            bins.clone(),
            bin_writer.clone(),
        )),
        Arc::new(emptied::BinsEmptied::new(bins, bin_writer)),
    ]
}
