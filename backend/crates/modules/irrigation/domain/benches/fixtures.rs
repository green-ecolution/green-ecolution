use chrono::{DateTime, TimeZone, Utc};
use domain::{Id, sensor::data::Watermark, shared::coordinates::Coordinate};
use irrigation_domain::{
    cluster::{SoilCondition, TreeCluster, TreeClusterSnapshot},
    tree::{Tree, snapshot::TreeSnapshot},
    watering_status::WateringStatus,
};
use uuid::Uuid;

/// Benches are an external crate, so the aggregate's private fields are out of
/// reach; `reconstitute` is the only constructor available here.
pub fn cluster_with_trees(tree_count: usize) -> TreeCluster {
    TreeCluster::reconstitute(TreeClusterSnapshot {
        id: Uuid::now_v7(),
        name: "Benchmark-Cluster".to_string(),
        address: "Stadtpark 1, 24937 Flensburg".to_string(),
        description: "Benchmark".to_string(),
        watering_status: WateringStatus::Unknown,
        last_watered: None,
        moisture_level: 0.5,
        region_id: None,
        archived: false,
        latitude: Some(54.7937),
        longitude: Some(9.4469),
        soil_condition: Some(SoilCondition::Lu),
        tree_ids: (0..tree_count).map(|_| Uuid::now_v7()).collect(),
        provider: None,
        additional_info: None,
        organization_id: Uuid::now_v7(),
    })
}

/// Spread around the Flensburg city centre rather than over a wide rectangle,
/// so the centroid maths sees the magnitudes it sees in production.
pub fn coordinates(n: usize) -> Vec<Coordinate> {
    (0..n)
        .map(|i| {
            let offset = i as f64 * 0.000_01;
            Coordinate::new(54.7937 + offset, 9.4469 + offset)
                .expect("offsets stay well inside the valid lat/lon range")
        })
        .collect()
}

pub fn statuses(n: usize) -> Vec<WateringStatus> {
    let cycle = [
        WateringStatus::Good,
        WateringStatus::Moderate,
        WateringStatus::Bad,
        WateringStatus::JustWatered,
        WateringStatus::Unknown,
    ];
    (0..n).map(|i| cycle[i % cycle.len()]).collect()
}

pub fn tree_ids(n: usize) -> Vec<Id<Tree>> {
    (0..n).map(|_| Id::new_v7()).collect()
}

pub fn fixed_today() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 1, 12, 0, 0)
        .single()
        .expect("a fixed, unambiguous timestamp")
}

pub fn tree_planted_in(year: i32) -> Tree {
    Tree::reconstitute(TreeSnapshot {
        id: Uuid::now_v7(),
        cluster_id: None,
        planting_year: year,
        species: "Tilia cordata".to_string(),
        tree_number: "A1001".to_string(),
        latitude: 54.7937,
        longitude: 9.4469,
        watering_status: WateringStatus::Unknown,
        description: None,
        last_watered: None,
        provider: None,
        additional_info: None,
        organization_id: Uuid::now_v7(),
    })
}

pub fn watermarks() -> Vec<Watermark> {
    vec![
        Watermark {
            depth: 30,
            resistance: 0,
            centibar: 40,
        },
        Watermark {
            depth: 60,
            resistance: 0,
            centibar: 55,
        },
        Watermark {
            depth: 90,
            resistance: 0,
            centibar: 70,
        },
    ]
}
