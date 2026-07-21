//! Read models for the cluster soil-moisture dashboard: bucketed time series
//! aggregated across all sensors of a cluster, plus finished watering runs.

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::tree::VolumetricThresholds;

/// Aggregation bucket for the soil-moisture time series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoilMoistureBucket {
    Hour,
    Day,
}

/// One aggregated bucket: mean/min/max over every in-range reading of every
/// cluster sensor at one depth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoilMoisturePoint {
    pub bucket_start: DateTime<Utc>,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoilMoistureDepthSeries {
    pub depth_cm: i32,
    pub points: Vec<SoilMoisturePoint>,
}

/// A finished watering-plan run that included this cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClusterWateringEvent {
    pub watering_plan_id: Uuid,
    pub date: NaiveDate,
    pub consumed_water_liters: f64,
}

/// Everything the cluster dashboard chart needs in one read.
#[derive(Debug, Clone, PartialEq)]
pub struct SoilMoistureOverview {
    pub bucket: SoilMoistureBucket,
    pub series: Vec<SoilMoistureDepthSeries>,
    pub thresholds: Vec<VolumetricThresholds>,
    pub watering_events: Vec<ClusterWateringEvent>,
}
