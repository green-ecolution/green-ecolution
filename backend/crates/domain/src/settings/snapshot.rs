use uuid::Uuid;

/// Raw row mapping used exclusively for rehydration.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OrganizationSettingsSnapshot {
    pub organization_id: Uuid,
    pub water_demand_liters: Option<f64>,
    pub just_watered_ttl_secs: Option<i64>,
    pub sensor_offline_after_secs: Option<i64>,
    pub defect_streak: Option<i32>,
    pub map_center_lat: Option<f64>,
    pub map_center_lng: Option<f64>,
    pub map_bbox_sw_lat: Option<f64>,
    pub map_bbox_sw_lng: Option<f64>,
    pub map_bbox_ne_lat: Option<f64>,
    pub map_bbox_ne_lng: Option<f64>,
    pub descendants_may_override: bool,
}
