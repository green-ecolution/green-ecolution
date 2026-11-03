use std::collections::BTreeSet;

use crate::{Id, authorization::OrgHierarchy, organization::Organization};

/// The organization on `org`'s path to the root that owns its export; the
/// nearest one wins, whether its target is enabled or not.
pub fn responsible_target(
    hierarchy: &OrgHierarchy,
    targets: &BTreeSet<Id<Organization>>,
    org: Id<Organization>,
) -> Option<Id<Organization>> {
    hierarchy
        .ancestors_from_root(org)
        .into_iter()
        .rev()
        .find(|candidate| targets.contains(candidate))
}

/// Every organization whose objects go to `target_org`'s target.
pub fn export_scope(
    hierarchy: &OrgHierarchy,
    targets: &BTreeSet<Id<Organization>>,
    target_org: Id<Organization>,
) -> BTreeSet<Id<Organization>> {
    hierarchy
        .descendants_or_self(target_org)
        .into_iter()
        .filter(|org| responsible_target(hierarchy, targets, *org) == Some(target_org))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn id(n: u128) -> Id<Organization> {
        Id::new(Uuid::from_u128(n))
    }

    // root(1) ── tenant(2) ── unit(3)
    //         └─ tenant(4)
    fn tree() -> OrgHierarchy {
        OrgHierarchy::from_pairs([
            (id(1), None),
            (id(2), Some(id(1))),
            (id(3), Some(id(2))),
            (id(4), Some(id(1))),
        ])
    }

    #[test]
    fn the_nearest_target_on_the_way_up_is_responsible() {
        let targets = BTreeSet::from([id(1), id(2)]);
        assert_eq!(responsible_target(&tree(), &targets, id(3)), Some(id(2)));
        assert_eq!(responsible_target(&tree(), &targets, id(4)), Some(id(1)));
        assert_eq!(responsible_target(&tree(), &BTreeSet::new(), id(3)), None);
    }

    #[test]
    fn a_scope_stops_at_a_sub_tree_with_its_own_target() {
        let targets = BTreeSet::from([id(1), id(2)]);
        assert_eq!(
            export_scope(&tree(), &targets, id(1)),
            BTreeSet::from([id(1), id(4)])
        );
        assert_eq!(
            export_scope(&tree(), &targets, id(2)),
            BTreeSet::from([id(2), id(3)])
        );
    }
}
