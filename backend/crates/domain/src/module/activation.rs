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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationResolution {
    pub enabled: bool,
    pub origin: SettingOrigin,
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
        })
        .collect();
    let r = inherit(default_enabled, &levels, target);
    ActivationResolution {
        enabled: r.value,
        origin: r.origin,
        inherited: r.inherited,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(org: Id<Organization>, enabled: Option<bool>) -> ActivationLevel {
        ActivationLevel {
            organization_id: org,
            enabled,
        }
    }

    fn orgs() -> (Id<Organization>, Id<Organization>, Id<Organization>) {
        (Id::new_v7(), Id::new_v7(), Id::new_v7())
    }

    #[test]
    fn untouched_chain_yields_the_default() {
        let (root, mid, leaf) = orgs();
        let chain = [level(root, None), level(mid, None), level(leaf, None)];
        let r = resolve_activation(true, &chain, leaf);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Default);
    }

    #[test]
    fn value_set_at_the_root_reaches_three_levels_down() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(false)),
            level(mid, None),
            level(leaf, None),
        ];
        let r = resolve_activation(true, &chain, leaf);
        assert!(!r.enabled);
        assert_eq!(r.origin, SettingOrigin::Inherited(root));
    }

    #[test]
    fn own_value_beats_the_parent() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, Some(false)),
            level(mid, None),
            level(leaf, Some(true)),
        ];
        let r = resolve_activation(false, &chain, leaf);
        assert!(r.enabled);
        assert_eq!(r.origin, SettingOrigin::Own);
    }

    #[test]
    fn inherited_ignores_the_targets_own_switch() {
        let (root, mid, leaf) = orgs();
        let chain = [
            level(root, None),
            level(mid, Some(false)),
            level(leaf, Some(true)),
        ];
        let r = resolve_activation(true, &chain, leaf);
        assert!(r.enabled);
        assert!(!r.inherited);

        let root_only = [level(root, Some(true))];
        assert!(!resolve_activation(false, &root_only, root).inherited);
    }
}
