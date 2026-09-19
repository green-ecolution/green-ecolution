use std::collections::BTreeSet;

use chrono::{DateTime, TimeZone, Utc};
use domain::{
    Id,
    authorization::{Action, OrgHierarchy, Permission, Resource},
    cluster::{SoilCondition, TreeCluster, TreeClusterSnapshot},
    organization::Organization,
    sensor::data::Watermark,
    settings::{
        DefectStreak, InstanceDefaults, JustWateredTtl, MapView, OrganizationSettings,
        SensorOfflineAfter, WaterDemand,
    },
    shared::{coordinates::Coordinate, watering_status::WateringStatus},
    tree::{Tree, snapshot::TreeSnapshot},
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

/// One root with `children` direct children. The leaf is one of the children,
/// so an ancestry walk terminates after a single hop.
pub fn flat_hierarchy(children: usize) -> (OrgHierarchy, Id<Organization>, Id<Organization>) {
    let root: Id<Organization> = Id::new_v7();
    let mut pairs = vec![(root, None)];
    let mut leaf = root;
    for _ in 0..children {
        let child = Id::new_v7();
        pairs.push((child, Some(root)));
        leaf = child;
    }
    (OrgHierarchy::from_pairs(pairs), root, leaf)
}

/// A single chain of `depth` nodes. The leaf sits at the far end, so an
/// ancestry walk traverses every node, the worst case the flat shape hides.
pub fn deep_hierarchy(depth: usize) -> (OrgHierarchy, Id<Organization>, Id<Organization>) {
    let root: Id<Organization> = Id::new_v7();
    let mut pairs = vec![(root, None)];
    let mut current = root;
    for _ in 0..depth {
        let child = Id::new_v7();
        pairs.push((child, Some(current)));
        current = child;
    }
    (OrgHierarchy::from_pairs(pairs), root, current)
}

pub fn grants(count: usize) -> Vec<Grant> {
    let permissions: BTreeSet<Permission> = Permission::catalog().into_iter().collect();
    (0..count)
        .map(|_| (Id::new_v7(), permissions.clone()))
        .collect()
}

pub type Grant = (Id<Organization>, BTreeSet<Permission>);

/// A flat tree of `nodes` organizations plus `grant_count` grants that point
/// *into* it, and one org id from the tree to probe with. Grants on ids the
/// tree does not know would make every lookup a miss and measure nothing.
pub fn hierarchy_with_grants(
    nodes: usize,
    grant_count: usize,
) -> (OrgHierarchy, Vec<Grant>, Id<Organization>) {
    let root: Id<Organization> = Id::new_v7();
    let mut pairs = vec![(root, None)];
    let mut children = Vec::with_capacity(nodes);
    for _ in 0..nodes {
        let child = Id::new_v7();
        pairs.push((child, Some(root)));
        children.push(child);
    }

    let permissions: BTreeSet<Permission> = Permission::catalog().into_iter().collect();
    let grants = children
        .iter()
        .take(grant_count)
        .map(|org| (*org, permissions.clone()))
        .collect();

    let probe = *children.first().unwrap_or(&root);
    (OrgHierarchy::from_pairs(pairs), grants, probe)
}

pub fn single_permission() -> Permission {
    Permission::new(Resource::Tree, Action::Read)
}

pub fn instance_defaults() -> InstanceDefaults {
    InstanceDefaults {
        water_demand: WaterDemand::new(120.0).expect("120 l is inside 1..=1000"),
        just_watered_ttl: JustWateredTtl::new(86_400).expect("1 day is inside 3600..=1209600"),
        sensor_offline_after: SensorOfflineAfter::new(86_400)
            .expect("1 day is inside 3600..=2592000"),
        defect_streak: DefectStreak::new(3).expect("3 is inside 1..=100"),
        map_view: MapView::new(
            Coordinate::new(54.7937, 9.4469).expect("Flensburg is a valid coordinate"),
            None,
        )
        .expect("a centre without bounds is always valid"),
    }
}

/// Root first, target last, the order `resolve` contracts for. Every level sets
/// `water_demand`, so the fold has to walk the whole chain.
pub fn settings_chain(depth: usize) -> (Vec<OrganizationSettings>, Id<Organization>) {
    let mut chain = Vec::with_capacity(depth);
    let mut target = Id::new_v7();
    for i in 0..depth {
        let org: Id<Organization> = Id::new_v7();
        target = org;
        chain.push(OrganizationSettings {
            organization_id: org,
            water_demand: WaterDemand::new(100.0 + (i % 100) as f64).ok(),
            just_watered_ttl: None,
            sensor_offline_after: None,
            defect_streak: None,
            map_view: None,
            descendants_may_override: true,
        });
    }
    (chain, target)
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
        sensor_id: None,
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
