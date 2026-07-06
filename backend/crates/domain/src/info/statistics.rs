//! Aggregate counts of domain entities.

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub tree_count: i64,
    pub sensor_count: i64,
    pub vehicle_count: i64,
    pub cluster_count: i64,
    pub watering_plan_count: i64,
}
