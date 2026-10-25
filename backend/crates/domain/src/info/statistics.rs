//! Aggregate counts of domain entities.

use crate::module::{ModuleCount, ModuleKey};

/// What the core counts itself; module objects come from `ModuleStatistics`.
#[derive(Debug, Clone, Copy)]
pub struct CoreCounts {
    pub sensor_count: i64,
    pub vehicle_count: i64,
    pub plan_count: i64,
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub sensor_count: i64,
    pub vehicle_count: i64,
    pub plan_count: i64,
    pub modules: Vec<(ModuleKey, ModuleCount)>,
}
