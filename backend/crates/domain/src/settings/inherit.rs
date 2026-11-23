use crate::{Id, organization::Organization};

/// Where a resolved value came from. `Inherited` names the organization that
/// actually set it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingOrigin {
    Default,
    Inherited(Id<Organization>),
    Own,
}

/// One organization's contribution to an inherited value, root first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InheritLevel<T> {
    pub organization_id: Id<Organization>,
    pub value: Option<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inherited<T> {
    pub value: T,
    pub origin: SettingOrigin,
    /// What applies once the target gives up its own value.
    pub inherited: T,
}

/// `chain` runs from the root to `target`. A set value overrides what came
/// before it.
pub fn inherit<T: Clone>(
    default: T,
    chain: &[InheritLevel<T>],
    target: Id<Organization>,
) -> Inherited<T> {
    let mut inherited = default;
    let mut origin = SettingOrigin::Default;
    let mut own = None;
    for level in chain {
        let Some(value) = &level.value else { continue };
        if level.organization_id == target {
            own = Some(value.clone());
        } else {
            inherited = value.clone();
            origin = SettingOrigin::Inherited(level.organization_id);
        }
    }
    match own {
        Some(value) => Inherited {
            value,
            origin: SettingOrigin::Own,
            inherited,
        },
        None => Inherited {
            value: inherited.clone(),
            origin,
            inherited,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(org: Id<Organization>, value: Option<u32>) -> InheritLevel<u32> {
        InheritLevel {
            organization_id: org,
            value,
        }
    }

    fn orgs() -> (Id<Organization>, Id<Organization>, Id<Organization>) {
        (Id::new_v7(), Id::new_v7(), Id::new_v7())
    }

    #[test]
    fn untouched_chain_yields_the_default_twice() {
        let (root, mid, leaf) = orgs();
        let chain = [level(root, None), level(mid, None), level(leaf, None)];
        assert_eq!(
            inherit(7, &chain, leaf),
            Inherited {
                value: 7,
                origin: SettingOrigin::Default,
                inherited: 7
            }
        );
    }

    #[test]
    fn nearest_set_value_wins_and_names_its_origin() {
        let (root, mid, leaf) = orgs();
        let chain = [level(root, Some(1)), level(mid, Some(2)), level(leaf, None)];
        let r = inherit(7, &chain, leaf);
        assert_eq!((r.value, r.inherited), (2, 2));
        assert_eq!(r.origin, SettingOrigin::Inherited(mid));
    }

    #[test]
    fn an_own_value_wins_and_leaves_the_inherited_one_visible() {
        let (root, mid, leaf) = orgs();
        let chain = [level(root, Some(1)), level(mid, None), level(leaf, Some(3))];
        let r = inherit(7, &chain, leaf);
        assert_eq!((r.value, r.origin, r.inherited), (3, SettingOrigin::Own, 1));
    }

    #[test]
    fn without_a_value_above_the_inherited_one_is_the_default() {
        let (root, _, leaf) = orgs();
        let chain = [level(root, None), level(leaf, Some(3))];
        assert_eq!(inherit(7, &chain, leaf).inherited, 7);
    }
}
