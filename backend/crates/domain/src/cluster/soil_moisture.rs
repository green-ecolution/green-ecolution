//! Read models for the cluster soil-moisture dashboard: bucketed time series
//! aggregated across all sensors of a cluster, plus finished watering runs.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use super::soil_condition::SoilCondition;
use crate::tree::{VolumetricThresholds, rew_fraction};

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

/// One bucket of the tree-condition series: REW fractions of the depth that
/// is worst by mean. The min/max band comes from that same depth so the band
/// never mixes depths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoilMoistureConditionPoint {
    pub bucket_start: DateTime<Utc>,
    pub rew_mean: f64,
    pub rew_min: f64,
    pub rew_max: f64,
    pub worst_depth_cm: i32,
}

/// Worst-case condition series across depths: per bucket, the depth with the
/// lowest mean REW wins. Buckets where no depth is calibratable are dropped;
/// `Unknown` soil yields an empty series.
pub fn condition_series(
    series: &[SoilMoistureDepthSeries],
    soil: SoilCondition,
) -> Vec<SoilMoistureConditionPoint> {
    let mut by_bucket: BTreeMap<DateTime<Utc>, SoilMoistureConditionPoint> = BTreeMap::new();
    for depth in series {
        for p in &depth.points {
            let (Some(rew_mean), Some(rew_min), Some(rew_max)) = (
                rew_fraction(soil, depth.depth_cm, p.mean),
                rew_fraction(soil, depth.depth_cm, p.min),
                rew_fraction(soil, depth.depth_cm, p.max),
            ) else {
                continue;
            };
            let candidate = SoilMoistureConditionPoint {
                bucket_start: p.bucket_start,
                rew_mean,
                rew_min,
                rew_max,
                worst_depth_cm: depth.depth_cm,
            };
            by_bucket
                .entry(p.bucket_start)
                .and_modify(|current| {
                    if candidate.rew_mean < current.rew_mean {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }
    }
    by_bucket.into_values().collect()
}

/// Everything the cluster dashboard chart needs in one read.
#[derive(Debug, Clone, PartialEq)]
pub struct SoilMoistureOverview {
    pub bucket: SoilMoistureBucket,
    pub series: Vec<SoilMoistureDepthSeries>,
    pub thresholds: Vec<VolumetricThresholds>,
    /// Worst-case REW series; empty when the soil condition is unknown.
    pub condition: Vec<SoilMoistureConditionPoint>,
    pub watering_events: Vec<ClusterWateringEvent>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::cluster::SoilCondition;

    fn point(ts_hour: u32, mean: f64, min: f64, max: f64) -> SoilMoisturePoint {
        SoilMoisturePoint {
            bucket_start: Utc.with_ymd_and_hms(2026, 7, 2, ts_hour, 0, 0).unwrap(),
            mean,
            min,
            max,
            sample_count: 1,
        }
    }

    // Uu at both depths: PWP = 12, nFK_eff = 20.
    #[test]
    fn worst_depth_by_mean_wins_and_band_comes_from_that_depth() {
        let series = vec![
            SoilMoistureDepthSeries {
                depth_cm: 40,
                points: vec![point(0, 25.0, 24.0, 26.0)], // REW mean 0.65
            },
            SoilMoistureDepthSeries {
                depth_cm: 80,
                points: vec![point(0, 15.0, 14.0, 16.0)], // REW mean 0.15
            },
        ];
        let condition = condition_series(&series, SoilCondition::Uu);
        assert_eq!(condition.len(), 1);
        let c = condition[0];
        assert_eq!(c.worst_depth_cm, 80);
        assert!((c.rew_mean - 0.15).abs() < 1e-9);
        assert!((c.rew_min - 0.10).abs() < 1e-9);
        assert!((c.rew_max - 0.20).abs() < 1e-9);
    }

    #[test]
    fn buckets_missing_in_one_depth_still_appear_sorted() {
        let series = vec![
            SoilMoistureDepthSeries {
                depth_cm: 40,
                points: vec![point(2, 25.0, 25.0, 25.0)],
            },
            SoilMoistureDepthSeries {
                depth_cm: 80,
                points: vec![point(1, 15.0, 15.0, 15.0)],
            },
        ];
        let condition = condition_series(&series, SoilCondition::Uu);
        assert_eq!(condition.len(), 2);
        assert_eq!(condition[0].worst_depth_cm, 80);
        assert_eq!(condition[1].worst_depth_cm, 40);
        assert!(condition[0].bucket_start < condition[1].bucket_start);
    }

    #[test]
    fn unknown_soil_yields_empty_series() {
        let series = vec![SoilMoistureDepthSeries {
            depth_cm: 40,
            points: vec![point(0, 25.0, 25.0, 25.0)],
        }];
        assert!(condition_series(&series, SoilCondition::Unknown).is_empty());
    }

    #[test]
    fn uncalibrated_depth_is_skipped() {
        let series = vec![
            SoilMoistureDepthSeries {
                depth_cm: 55, // no calibration data
                points: vec![point(0, 5.0, 5.0, 5.0)],
            },
            SoilMoistureDepthSeries {
                depth_cm: 40,
                points: vec![point(0, 25.0, 25.0, 25.0)],
            },
        ];
        let condition = condition_series(&series, SoilCondition::Uu);
        assert_eq!(condition.len(), 1);
        assert_eq!(condition[0].worst_depth_cm, 40);
    }
}
