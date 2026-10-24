//! References from the core to objects a module owns, such as a tree or a
//! waste bin. The core never interprets the id; it asks the owning module's
//! catalog.

use std::{collections::HashMap, fmt};

use uuid::Uuid;

use crate::{
    Id, RepositoryError,
    authorization::ResourceKey,
    module::ModuleKey,
    organization::Organization,
    shared::{coordinates::Coordinate, error::ValidationError, key::validate_key},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetKind(String);

impl TargetKind {
    const FIELD: &'static str = "target.kind";

    pub fn new(value: &str) -> Result<Self, ValidationError> {
        validate_key(Self::FIELD, value)?;
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetRef {
    pub module: ModuleKey,
    pub kind: TargetKind,
    pub id: Uuid,
}

impl TargetRef {
    pub fn new(module: &str, kind: &str, id: Uuid) -> Result<Self, ValidationError> {
        Ok(Self {
            module: ModuleKey::new(module)?,
            kind: TargetKind::new(kind)?,
            id,
        })
    }
}

/// What a module allows for one kind of its objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetKindSpec {
    pub kind: &'static str,
    pub accepts_sensors: bool,
    pub commentable: bool,
    /// Permissions on the target are checked against this resource.
    pub resource: ResourceKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPurpose {
    Sensor,
    Comment,
}

impl TargetKindSpec {
    pub fn allows(&self, purpose: TargetPurpose) -> bool {
        match purpose {
            TargetPurpose::Sensor => self.accepts_sensors,
            TargetPurpose::Comment => self.commentable,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TargetError {
    #[error("target not found")]
    NotFound,
    /// A module-specific rule refuses the target; `code` is the stable error
    /// code the client sees.
    #[error("{message}")]
    Refused { code: &'static str, message: String },
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[async_trait::async_trait]
pub trait TargetCatalog: Send + Sync {
    fn kinds(&self) -> &'static [TargetKindSpec];

    /// Checks that the target exists and may serve `purpose`, and returns the
    /// organization it belongs to.
    async fn admit(
        &self,
        target: &TargetRef,
        purpose: TargetPurpose,
    ) -> Result<Id<Organization>, TargetError>;

    /// Targets without a position are left out of the map.
    async fn locate(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, Coordinate>, RepositoryError>;

    async fn label(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, String>, RepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_ref_validates_both_keys() {
        assert!(TargetRef::new("irrigation", "tree", Uuid::nil()).is_ok());
        assert!(TargetRef::new("irrigation", "Tree", Uuid::nil()).is_err());
        assert!(TargetRef::new("", "tree", Uuid::nil()).is_err());
    }

    #[test]
    fn spec_allows_by_purpose() {
        let spec = TargetKindSpec {
            kind: "tree",
            accepts_sensors: true,
            commentable: false,
            resource: ResourceKey::declare("tree"),
        };
        assert!(spec.allows(TargetPurpose::Sensor));
        assert!(!spec.allows(TargetPurpose::Comment));
    }
}
