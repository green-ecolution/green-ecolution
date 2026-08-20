use uuid::Uuid;

use crate::shared::watering_status::WateringStatus;

/// Lightweight projection of a tree intended for map markers.
///
/// Carries only what the marker layer renders: position, status, label,
/// whether a sensor exists, and the owning organization — the cluster forms
/// need it to tell selectable trees from foreign ones. It is *not* an
/// aggregate hydration target.
#[derive(Debug, Clone)]
pub struct TreeMarker {
    pub id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub watering_status: WateringStatus,
    pub tree_number: String,
    pub has_sensor: bool,
    pub organization_id: Uuid,
}
