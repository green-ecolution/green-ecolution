use std::str::FromStr;

use crate::shared::error::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Resource {
    Tree,
    TreeCluster,
    Sensor,
    WateringPlan,
    Vehicle,
    Region,
    User,
    Organization,
    Role,
}

impl Resource {
    pub const ALL: [Resource; 9] = [
        Resource::Tree,
        Resource::TreeCluster,
        Resource::Sensor,
        Resource::WateringPlan,
        Resource::Vehicle,
        Resource::Region,
        Resource::User,
        Resource::Organization,
        Resource::Role,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Resource::Tree => "tree",
            Resource::TreeCluster => "tree_cluster",
            Resource::Sensor => "sensor",
            Resource::WateringPlan => "watering_plan",
            Resource::Vehicle => "vehicle",
            Resource::Region => "region",
            Resource::User => "user",
            Resource::Organization => "organization",
            Resource::Role => "role",
        }
    }
}

impl FromStr for Resource {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Resource::ALL
            .into_iter()
            .find(|r| r.as_str() == s)
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "permission",
                reason: format!("unknown resource '{s}'"),
            })
    }
}

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
                field: "permission",
                reason: format!("unknown action '{s}'"),
            })
    }
}

/// A single grantable capability, e.g. `tree:read`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}

impl Permission {
    pub fn new(resource: Resource, action: Action) -> Self {
        Self { resource, action }
    }

    pub fn catalog() -> Vec<Permission> {
        Resource::ALL
            .into_iter()
            .flat_map(|r| Action::ALL.into_iter().map(move |a| Permission::new(r, a)))
            .collect()
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource.as_str(), self.action.as_str())
    }
}

impl FromStr for Permission {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (res, act) = s
            .split_once(':')
            .ok_or_else(|| ValidationError::InvalidFormat {
                field: "permission",
                reason: format!("expected '<resource>:<action>', got '{s}'"),
            })?;
        Ok(Permission::new(
            Resource::from_str(res)?,
            Action::from_str(act)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_err;

    #[test]
    fn roundtrips_every_catalog_entry() {
        for p in Permission::catalog() {
            let parsed = Permission::from_str(&p.to_string()).unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn display_uses_snake_case_resource_and_colon() {
        let p = Permission::new(Resource::TreeCluster, Action::Read);
        assert_eq!(p.to_string(), "tree_cluster:read");
    }

    #[test]
    fn rejects_unknown_resource() {
        assert_err!(Permission::from_str("garden:read"));
    }

    #[test]
    fn rejects_unknown_action() {
        assert_err!(Permission::from_str("tree:fly"));
    }

    #[test]
    fn rejects_missing_separator() {
        assert_err!(Permission::from_str("treeread"));
    }

    #[test]
    fn catalog_has_all_combinations() {
        assert_eq!(Permission::catalog().len(), 36);
    }
}
