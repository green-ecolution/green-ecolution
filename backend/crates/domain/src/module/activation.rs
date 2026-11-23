/// A module's state in one organization: what the root allows and what the
/// tenant chose. Both live on the tenant's row; sub-organizations follow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleState {
    pub released: bool,
    pub enabled: bool,
}

impl ModuleState {
    /// Usable in the organization: released by the root and switched on.
    pub fn active(self) -> bool {
        self.released && self.enabled
    }
}

/// Whose row decides for an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchHolder {
    /// The root is no tenant: always released, switched by `default_enabled`
    /// until it stores a switch of its own.
    Root,
    /// A tenant without a row has the module locked.
    Tenant,
}

pub fn module_state(
    holder: SwitchHolder,
    default_enabled: bool,
    row: Option<ModuleState>,
) -> ModuleState {
    match (holder, row) {
        (SwitchHolder::Root, row) => ModuleState {
            released: true,
            enabled: row.map_or(default_enabled, |r| r.enabled),
        },
        (SwitchHolder::Tenant, Some(row)) => row,
        (SwitchHolder::Tenant, None) => ModuleState {
            released: false,
            enabled: false,
        },
    }
}

/// The row after releasing: the first release switches the module on, a
/// later one restores what the tenant had when it was locked.
pub fn released_state(row: Option<ModuleState>) -> ModuleState {
    ModuleState {
        released: true,
        enabled: row.is_none_or(|r| r.enabled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ON: ModuleState = ModuleState {
        released: true,
        enabled: true,
    };
    const OFF: ModuleState = ModuleState {
        released: true,
        enabled: false,
    };

    #[test]
    fn the_root_is_always_released_and_follows_the_default_without_a_row() {
        assert_eq!(module_state(SwitchHolder::Root, true, None), ON);
        assert_eq!(module_state(SwitchHolder::Root, false, None), OFF);
    }

    #[test]
    fn the_roots_own_switch_beats_the_default() {
        assert_eq!(module_state(SwitchHolder::Root, true, Some(OFF)), OFF);
        let locked_row = ModuleState {
            released: false,
            enabled: true,
        };
        assert!(module_state(SwitchHolder::Root, false, Some(locked_row)).released);
    }

    #[test]
    fn a_tenant_without_a_row_is_locked_whatever_the_default() {
        let state = module_state(SwitchHolder::Tenant, true, None);
        assert!(!state.released);
        assert!(!state.active());
    }

    #[test]
    fn a_released_module_is_active_only_while_switched_on() {
        assert!(module_state(SwitchHolder::Tenant, false, Some(ON)).active());
        assert!(!module_state(SwitchHolder::Tenant, true, Some(OFF)).active());
    }

    #[test]
    fn a_locked_module_is_inactive_even_while_switched_on() {
        let locked = ModuleState {
            released: false,
            enabled: true,
        };
        assert!(!module_state(SwitchHolder::Tenant, true, Some(locked)).active());
    }

    #[test]
    fn the_first_release_switches_on_and_a_later_one_restores() {
        assert_eq!(released_state(None), ON);
        let locked_off = ModuleState {
            released: false,
            enabled: false,
        };
        assert_eq!(released_state(Some(locked_off)), OFF);
        let locked_on = ModuleState {
            released: false,
            enabled: true,
        };
        assert_eq!(released_state(Some(locked_on)), ON);
    }
}
