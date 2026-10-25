use std::fmt;

use crate::{
    authorization::{Action, Permission},
    shared::{error::ValidationError, key::validate_key},
};

/// Stable name of something permissions are granted on. Core resources are
/// constants here; modules declare theirs as constants too, and the catalog
/// validates every declared key once at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceKey(&'static str);

impl ResourceKey {
    pub const SENSOR: Self = Self("sensor");
    pub const VEHICLE: Self = Self("vehicle");
    pub const REGION: Self = Self("region");
    pub const USER: Self = Self("user");
    pub const ORGANIZATION: Self = Self("organization");
    pub const ROLE: Self = Self("role");
    pub const PLUGIN: Self = Self("plugin");
    pub const SETTING: Self = Self("setting");
    pub const START_POINT: Self = Self("start_point");
    pub const CORE: [Self; 9] = [
        Self::SENSOR,
        Self::VEHICLE,
        Self::REGION,
        Self::USER,
        Self::ORGANIZATION,
        Self::ROLE,
        Self::PLUGIN,
        Self::SETTING,
        Self::START_POINT,
    ];

    /// Unchecked so modules can declare keys as constants.
    pub const fn declare(key: &'static str) -> Self {
        Self(key)
    }

    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ResourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PermissionParseError {
    #[error(transparent)]
    Malformed(#[from] ValidationError),
    #[error("unknown resource '{0}'")]
    UnknownResource(String),
}

/// Every resource this build knows: the core first, then the registered
/// modules in registration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCatalog {
    resources: Vec<ResourceKey>,
}

const FIELD: &str = "authorization.resource";

impl ResourceCatalog {
    pub fn new(
        module_resources: impl IntoIterator<Item = ResourceKey>,
    ) -> Result<Self, ValidationError> {
        let mut resources = ResourceKey::CORE.to_vec();
        for key in module_resources {
            validate_key(FIELD, key.as_str())?;
            if resources.contains(&key) {
                return Err(ValidationError::InvalidFormat {
                    field: FIELD,
                    reason: format!("resource '{key}' is declared twice"),
                });
            }
            resources.push(key);
        }
        Ok(Self { resources })
    }

    pub fn core() -> Self {
        Self {
            resources: ResourceKey::CORE.to_vec(),
        }
    }

    pub fn resources(&self) -> &[ResourceKey] {
        &self.resources
    }

    pub fn resolve(&self, key: &str) -> Option<ResourceKey> {
        self.resources.iter().copied().find(|r| r.as_str() == key)
    }

    pub fn permissions(&self) -> Vec<Permission> {
        self.resources
            .iter()
            .flat_map(|r| Action::ALL.iter().map(move |a| Permission::new(*r, *a)))
            .collect()
    }

    pub fn parse_permission(&self, s: &str) -> Result<Permission, PermissionParseError> {
        let (resource, action) =
            s.split_once(':')
                .ok_or_else(|| ValidationError::InvalidFormat {
                    field: "authorization.permission",
                    reason: format!("expected '<resource>:<action>', got '{s}'"),
                })?;
        let action: Action = action.parse()?;
        let resource = self
            .resolve(resource)
            .ok_or_else(|| PermissionParseError::UnknownResource(resource.to_string()))?;
        Ok(Permission::new(resource, action))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDGET: ResourceKey = ResourceKey::declare("widget");

    #[test]
    fn core_catalog_has_nine_resources() {
        assert_eq!(ResourceCatalog::core().resources().len(), 9);
    }

    #[test]
    fn module_resources_follow_the_core() {
        let catalog = ResourceCatalog::new([WIDGET]).unwrap();
        assert_eq!(catalog.resources().len(), 10);
        assert_eq!(catalog.resources()[9], WIDGET);
    }

    #[test]
    fn permissions_are_resources_times_actions() {
        let catalog = ResourceCatalog::new([WIDGET]).unwrap();
        assert_eq!(catalog.permissions().len(), 10 * 4);
    }

    #[test]
    fn rejects_an_invalid_module_key() {
        assert!(ResourceCatalog::new([ResourceKey::declare("Widget")]).is_err());
    }

    #[test]
    fn rejects_a_key_declared_twice() {
        assert!(ResourceCatalog::new([WIDGET, WIDGET]).is_err());
        assert!(ResourceCatalog::new([ResourceKey::SENSOR]).is_err());
    }

    #[test]
    fn parses_known_permissions() {
        let catalog = ResourceCatalog::new([WIDGET]).unwrap();
        assert_eq!(
            catalog.parse_permission("widget:update").unwrap(),
            Permission::new(WIDGET, Action::Update)
        );
    }

    #[test]
    fn unknown_resource_is_its_own_error() {
        let catalog = ResourceCatalog::core();
        assert_eq!(
            catalog.parse_permission("widget:read"),
            Err(PermissionParseError::UnknownResource("widget".into()))
        );
    }

    #[test]
    fn malformed_strings_are_malformed() {
        let catalog = ResourceCatalog::core();
        assert!(matches!(
            catalog.parse_permission("sensor"),
            Err(PermissionParseError::Malformed(_))
        ));
        assert!(matches!(
            catalog.parse_permission("sensor:fly"),
            Err(PermissionParseError::Malformed(_))
        ));
    }
}
