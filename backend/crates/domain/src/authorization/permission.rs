use std::str::FromStr;

use crate::{authorization::ResourceKey, shared::error::ValidationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Read,
    Create,
    Update,
    Delete,
}

impl Action {
    pub const ALL: [Action; 4] = [Action::Read, Action::Create, Action::Update, Action::Delete];

    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Create => "create",
            Action::Update => "update",
            Action::Delete => "delete",
        }
    }
}

impl FromStr for Action {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Action::ALL
            .into_iter()
            .find(|a| a.as_str() == s)
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "authorization.permission",
                reason: format!("unknown action '{s}'"),
            })
    }
}

/// A single grantable capability, e.g. `tree:read`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Permission {
    pub resource: ResourceKey,
    pub action: Action,
}

impl Permission {
    pub fn new(resource: ResourceKey, action: Action) -> Self {
        Self { resource, action }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource.as_str(), self.action.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err;

    const TREE_CLUSTER: ResourceKey = ResourceKey::declare("tree_cluster");

    #[test]
    fn display_uses_snake_case_resource_and_colon() {
        let p = Permission::new(TREE_CLUSTER, Action::Read);
        assert_eq!(p.to_string(), "tree_cluster:read");
    }

    #[test]
    fn rejects_unknown_action() {
        assert_err!(Action::from_str("fly"));
    }

    #[test]
    fn roundtrips_every_core_permission() {
        let catalog = crate::authorization::ResourceCatalog::core();
        for p in catalog.permissions() {
            assert_eq!(catalog.parse_permission(&p.to_string()).unwrap(), p);
        }
    }
}
