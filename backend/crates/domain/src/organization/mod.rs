//! Organization aggregate — a node in the tenant/organization tree.
//!
//! Exactly one root row exists (`parent_id = NULL`, seeded by migration,
//! enforced by a partial unique index); its direct children are tenants.
//! Invariants: sibling-unique names (DB unique on `(parent_id, name)`), the
//! root can never be renamed or deleted, deletion requires no children and no
//! assigned users (checked in the service, backstopped by `ON DELETE
//! RESTRICT`). There is deliberately no move operation.

pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use crate::{Id, events::DomainEvent};

pub use error::OrganizationError;
pub use repository::{OrganizationReader, OrganizationWriter};
#[doc(hidden)]
pub use snapshot::OrganizationSnapshot;
pub use view::OrganizationView;

crate::newtype_nonempty! {
    /// Organization display name, 1–120 characters after trimming.
    OrganizationName, "organization.name", 1, 120
}

/// Well-known id of the root organization; identical in every environment.
pub fn root_organization_id() -> Id<Organization> {
    Id::new(crate::RawId::from_u128(0x0198_0000_0000_7000_8000_0000_0000_0001))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: Id<Organization>,
    pub name: OrganizationName,
    parent_id: Option<Id<Organization>>,
}

/// Input for creating a new organization. The root exists only via migration,
/// so a parent is always required.
#[derive(Debug, Clone)]
pub struct OrganizationDraft {
    pub name: OrganizationName,
    pub parent_id: Id<Organization>,
}

impl Organization {
    #[doc(hidden)]
    pub fn reconstitute(snap: OrganizationSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            name: OrganizationName::reconstitute(snap.name),
            parent_id: snap.parent_id.map(Id::new),
        }
    }

    pub fn parent_id(&self) -> Option<Id<Organization>> {
        self.parent_id
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn rename(
        &mut self,
        new_name: OrganizationName,
    ) -> Result<Vec<DomainEvent>, OrganizationError> {
        if self.is_root() {
            return Err(OrganizationError::RootImmutable);
        }
        if self.name == new_name {
            return Ok(Vec::new());
        }
        self.name = new_name;
        Ok(vec![DomainEvent::OrganizationRenamed {
            organization_id: self.id,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn org(parent: Option<Id<Organization>>) -> Organization {
        Organization {
            id: Id::new_v7(),
            name: OrganizationName::new("TBZ Flensburg").unwrap(),
            parent_id: parent,
        }
    }

    #[test]
    fn name_rejects_empty() {
        assert_err!(OrganizationName::new("  "));
    }

    #[test]
    fn rename_emits_event() {
        let mut o = org(Some(Id::new_v7()));
        let events = o.rename(OrganizationName::new("TBZ").unwrap()).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            crate::events::DomainEvent::OrganizationRenamed { .. }
        ));
        assert_eq!(o.name.as_str(), "TBZ");
    }

    #[test]
    fn rename_to_same_is_noop_without_event() {
        let mut o = org(Some(Id::new_v7()));
        let events = o
            .rename(OrganizationName::new("TBZ Flensburg").unwrap())
            .unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn root_cannot_be_renamed() {
        let mut root = org(None);
        assert!(root.is_root());
        let result = root.rename(OrganizationName::new("Anders").unwrap());
        assert_err!(&result);
        assert!(matches!(
            result.unwrap_err(),
            OrganizationError::RootImmutable
        ));
    }

    #[test]
    fn reconstitute_maps_snapshot() {
        let id = uuid::Uuid::now_v7();
        let parent = uuid::Uuid::now_v7();
        let o = Organization::reconstitute(snapshot::OrganizationSnapshot {
            id,
            parent_id: Some(parent),
            name: "GaLaBau Müller".into(),
        });
        assert_eq!(o.id.value(), id);
        assert_eq!(o.parent_id().unwrap().value(), parent);
        assert_ok!(OrganizationName::new(o.name.as_str()));
    }
}
