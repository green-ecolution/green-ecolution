use uuid::Uuid;

use crate::shared::watering_status::WateringStatus;

/// Lightweight projection of a tree cluster intended for map markers.
///
/// Only clusters with a centroid (`latitude`/`longitude` non-null) are emitted
/// — clusters without member trees have no place to render.
#[derive(Debug, Clone)]
pub struct ClusterMarker {
    pub id: Uuid,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub watering_status: WateringStatus,
    pub tree_count: u32,
}
