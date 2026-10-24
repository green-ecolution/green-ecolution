use crate::{Id, organization::Organization, settings::SettingOrigin};

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
}

/// Same fold as `settings::resolve`: `chain` runs from the root to `target`,
/// a set value overrides what came before, and the topmost lock above the
/// target ends the walk so nothing below it can override.
pub fn resolve_activation(
    default_enabled: bool,
    chain: &[ActivationLevel],
    target: Id<Organization>,
) -> ActivationResolution {
    let mut resolution = ActivationResolution {
        enabled: default_enabled,
        origin: SettingOrigin::Default,
        enforced_by: None,
    };
    for level in chain {
        if let Some(enabled) = level.enabled {
            resolution.enabled = enabled;
            resolution.origin = if level.organization_id == target {
                SettingOrigin::Own
            } else {
                SettingOrigin::Inherited(level.organization_id)
            };
        }
        if !level.descendants_may_override && level.organization_id != target {
            resolution.enforced_by = Some(level.organization_id);
            break;
        }
    }
    resolution
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
    fn lock_at_the_target_does_not_bind_the_target_itself() {
        let (root, mid, _) = orgs();
        let chain = [level(root, None, true), level(mid, Some(true), false)];
        let r = resolve_activation(false, &chain, mid);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Own);
        assert_eq!(r.enforced_by, None);
    }
}
