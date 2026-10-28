use crate::{
    Id,
    organization::Organization,
    settings::{InheritLevel, SettingOrigin, inherit},
};

/// One organization's contribution to a module's activation, root first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationLevel {
    pub organization_id: Id<Organization>,
    pub enabled: Option<bool>,
    pub descendants_may_override: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationResolution {
    pub enabled: bool,
    pub origin: SettingOrigin,
    /// The topmost ancestor that froze this subtree, if any.
    pub enforced_by: Option<Id<Organization>>,
    /// The state without the target's own switch: what returning it to
    /// inheritance would lead to.
    pub inherited: bool,
}

/// An organization's own switch and the level it was set from. A `set_by`
/// above the organization is a parent's choice for this sub-unit, which only
/// someone acting for that level or above may change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnSwitch {
    pub enabled: bool,
    pub set_by: Id<Organization>,
}

/// Inherits like every setting, see `settings::inherit`.
pub fn resolve_activation(
    default_enabled: bool,
    chain: &[ActivationLevel],
    target: Id<Organization>,
) -> ActivationResolution {
    let levels: Vec<InheritLevel<bool>> = chain
        .iter()
        .map(|l| InheritLevel {
            organization_id: l.organization_id,
            value: l.enabled,
            descendants_may_override: l.descendants_may_override,
        })
        .collect();
    let r = inherit(default_enabled, &levels, target);
    let without_own: Vec<InheritLevel<bool>> = levels
        .iter()
        .map(|l| InheritLevel {
            value: if l.organization_id == target {
                None
            } else {
                l.value
            },
            ..*l
        })
        .collect();
    ActivationResolution {
        enabled: r.value,
        origin: r.origin,
        enforced_by: r.enforced_by,
        inherited: inherit(default_enabled, &without_own, target).value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(org: Id<Organization>, enabled: Option<bool>, may_override: bool) -> ActivationLevel {
        ActivationLevel {
            organization_id: org,
            enabled,
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
        let r = resolve_activation(true, &chain, leaf);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Default);
        assert_eq!(r.enforced_by, None);
    }

    #[test]
    fn value_set_at_the_root_reaches_three_levels_down() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(false), true),
            level(mid, None, true),
            level(leaf, None, true),
        ];
        let r = resolve_activation(true, &chain, leaf);
        assert!(!r.enabled);
        assert_eq!(r.origin, SettingOrigin::Inherited(root));
    }

    #[test]
    fn own_value_beats_the_parent() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(false), true),
            level(mid, None, true),
            level(leaf, Some(true), true),
        ];
        let r = resolve_activation(false, &chain, leaf);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Own);
    }

    #[test]
    fn lock_above_the_target_ignores_lower_values() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(true), false),
            level(mid, Some(false), true),
            level(leaf, Some(false), true),
        ];
        let r = resolve_activation(false, &chain, leaf);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Inherited(root));
        assert_eq!(r.enforced_by, Some(root));
    }

    #[test]
    fn inherited_ignores_the_targets_own_switch() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, None, true),
            level(mid, Some(false), true),
            level(leaf, Some(true), true),
        ];
        let r = resolve_activation(true, &chain, leaf);
        assert!(r.enabled);
        assert!(!r.inherited);

        let root_only = [level(root, Some(true), true)];
        assert!(!resolve_activation(false, &root_only, root).inherited);
    }

    #[test]
    fn inherited_under_a_lock_is_the_locked_value() {
        let (root, _, leaf) = orgs();
        let chain = [
            level(root, Some(false), false),
            level(leaf, Some(true), true),
        ];
        let r = resolve_activation(true, &chain, leaf);
        assert!(!r.enabled);
        assert!(!r.inherited);
    }

    #[test]
    fn lock_at_the_target_does_not_bind_the_target_itself() {
        let (root, mid, _) = orgs();
        let chain = [level(root, None, true), level(mid, Some(true), false)];
        let r = resolve_activation(false, &chain, mid);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Own);
        assert_eq!(r.enforced_by, None);
    }
}
