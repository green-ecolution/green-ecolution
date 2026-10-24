use crate::{Id, organization::Organization, settings::SettingOrigin};

/// One organization's contribution to an inherited value, root first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InheritLevel<T> {
    pub organization_id: Id<Organization>,
    pub value: Option<T>,
    pub descendants_may_override: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inherited<T> {
    pub value: T,
    pub origin: SettingOrigin,
    /// The topmost ancestor that froze this subtree, if any.
    pub enforced_by: Option<Id<Organization>>,
}

/// `chain` runs from the root to `target`. A set value overrides what came
/// before; the topmost lock above the target ends the walk, so nothing below
/// it can override. The lock speaks about descendants, never about the
/// target itself.
pub fn inherit<T: Clone>(
    default: T,
    chain: &[InheritLevel<T>],
    target: Id<Organization>,
) -> Inherited<T> {
    let mut out = Inherited {
        value: default,
        origin: SettingOrigin::Default,
        enforced_by: None,
    };
    for level in chain {
        if let Some(value) = &level.value {
            out.value = value.clone();
            out.origin = if level.organization_id == target {
                SettingOrigin::Own
            } else {
                SettingOrigin::Inherited(level.organization_id)
            };
        }
        if !level.descendants_may_override && level.organization_id != target {
            out.enforced_by = Some(level.organization_id);
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(org: Id<Organization>, value: Option<u32>, may_override: bool) -> InheritLevel<u32> {
        InheritLevel {
            organization_id: org,
            value,
            descendants_may_override: may_override,
        }
    }

    fn orgs() -> (Id<Organization>, Id<Organization>, Id<Organization>) {
        (Id::new_v7(), Id::new_v7(), Id::new_v7())
    }

    #[test]
    fn untouched_chain_yields_the_default() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, None, true),
            level(mid, None, true),
            level(leaf, None, true),
        ];
        let r = inherit(7, &chain, leaf);
        assert_eq!(
            r,
            Inherited {
                value: 7,
                origin: SettingOrigin::Default,
                enforced_by: None
            }
        );
    }

    #[test]
    fn nearest_set_value_wins_and_names_its_origin() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(1), true),
            level(mid, Some(2), true),
            level(leaf, None, true),
        ];
        let r = inherit(7, &chain, leaf);
        assert_eq!(r.value, 2);
        assert_eq!(r.origin, SettingOrigin::Inherited(mid));
    }

    #[test]
    fn own_value_is_marked_own() {
        let (root, _, leaf) = orgs();
        let chain = [level(root, Some(1), true), level(leaf, Some(3), true)];
        let r = inherit(7, &chain, leaf);
        assert_eq!((r.value, r.origin), (3, SettingOrigin::Own));
    }

    #[test]
    fn topmost_lock_freezes_everything_below() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(1), false),
            level(mid, Some(2), false),
            level(leaf, Some(3), true),
        ];
        let r = inherit(7, &chain, leaf);
        assert_eq!(r.value, 1);
        assert_eq!(r.origin, SettingOrigin::Inherited(root));
        assert_eq!(r.enforced_by, Some(root));
    }

    #[test]
    fn lock_on_the_target_itself_does_not_stop_the_walk() {
        let (root, _, leaf) = orgs();
        let chain = [level(root, Some(1), true), level(leaf, Some(3), false)];
        let r = inherit(7, &chain, leaf);
        assert_eq!((r.value, r.enforced_by), (3, None));
    }
}
