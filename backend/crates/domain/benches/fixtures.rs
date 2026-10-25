use std::collections::BTreeSet;

use domain::{
    Id,
    authorization::{Action, OrgHierarchy, Permission, ResourceCatalog, ResourceKey},
    organization::Organization,
    settings::{DefectStreak, InstanceDefaults, MapView, OrganizationSettings, SensorOfflineAfter},
    shared::coordinates::Coordinate,
};

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
    let permissions: BTreeSet<Permission> = catalog_permissions();
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

    let permissions: BTreeSet<Permission> = catalog_permissions();
    let grants = children
        .iter()
        .take(grant_count)
        .map(|org| (*org, permissions.clone()))
        .collect();

    let probe = *children.first().unwrap_or(&root);
    (OrgHierarchy::from_pairs(pairs), grants, probe)
}

pub fn single_permission() -> Permission {
    Permission::new(ResourceKey::declare("widget"), Action::Read)
}

fn catalog_permissions() -> BTreeSet<Permission> {
    ResourceCatalog::new([
        ResourceKey::declare("widget"),
        ResourceKey::declare("widget_part"),
        ResourceKey::declare("widget_plan"),
    ])
    .unwrap()
    .permissions()
    .into_iter()
    .collect()
}

pub fn instance_defaults() -> InstanceDefaults {
    InstanceDefaults {
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
/// `sensor_offline_after`, so the fold has to walk the whole chain.
pub fn settings_chain(depth: usize) -> (Vec<OrganizationSettings>, Id<Organization>) {
    let mut chain = Vec::with_capacity(depth);
    let mut target = Id::new_v7();
    for i in 0..depth {
        let org: Id<Organization> = Id::new_v7();
        target = org;
        chain.push(OrganizationSettings {
            organization_id: org,
            sensor_offline_after: SensorOfflineAfter::new(3_600 + (i % 100) as i64 * 60).ok(),
            defect_streak: None,
            map_view: None,
            descendants_may_override: true,
        });
    }
    (chain, target)
}
