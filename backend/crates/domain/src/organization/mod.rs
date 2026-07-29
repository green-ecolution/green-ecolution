//! Organization aggregate — a node in the tenant/organization tree.
//!
//! Exactly one root exists; its direct children are tenants. Invariants:
//! sibling-unique names, the root can never be modified or deleted, deletion
//! requires no children and no assigned users (checked in the service). The
//! address is all-or-nothing: an organization either has a complete address or
//! none. There is deliberately no move operation.

pub mod detail_view;
pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use uuid::Uuid;

use crate::{Id, events::DomainEvent, shared::address::Address};

pub use detail_view::{ContactPersonView, OrganizationDetailView};
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
    Id::new(crate::RawId::from_u128(
        0x0198_0000_0000_7000_8000_0000_0000_0001,
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: Id<Organization>,
    pub name: OrganizationName,
    parent_id: Option<Id<Organization>>,
    address: Option<Address>,
    contact_person: Option<Uuid>,
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
            address: Address::reconstitute(snap.street, snap.postal_code, snap.city),
            contact_person: snap.contact_person_id,
        }
    }

    pub fn parent_id(&self) -> Option<Id<Organization>> {
        self.parent_id
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    pub fn contact_person(&self) -> Option<Uuid> {
        self.contact_person
    }

    /// Replaces the whole editable set. Only the rename is worth an event —
    /// nothing reacts to address or contact-person changes.
    pub fn replace_details(
        &mut self,
        name: OrganizationName,
        address: Option<Address>,
        contact_person: Option<Uuid>,
    ) -> Result<Vec<DomainEvent>, OrganizationError> {
        if self.is_root() {
            return Err(OrganizationError::RootImmutable);
        }
        let renamed = self.name != name;
        self.name = name;
        self.address = address;
        self.contact_person = contact_person;
        Ok(if renamed {
            vec![DomainEvent::OrganizationRenamed {
                organization_id: self.id,
            }]
        } else {
            Vec::new()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::address::{City, PostalCode, Street};
    use claims::{assert_err, assert_ok};

    fn org(parent: Option<Id<Organization>>) -> Organization {
        Organization {
            id: Id::new_v7(),
            name: OrganizationName::new("TBZ Flensburg").unwrap(),
            parent_id: parent,
            address: None,
            contact_person: None,
        }
    }

    fn address() -> Address {
        Address::new(
            Street::new("Nordergraben 12").unwrap(),
            PostalCode::new("24937").unwrap(),
            City::new("Flensburg").unwrap(),
        )
    }

    fn same_name(o: &Organization) -> OrganizationName {
        o.name.clone()
    }

    #[test]
    fn name_rejects_empty() {
        assert_err!(OrganizationName::new("  "));
    }

    #[test]
    fn renaming_emits_event() {
        let mut o = org(Some(Id::new_v7()));
        let events = o
            .replace_details(OrganizationName::new("TBZ").unwrap(), None, None)
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            crate::events::DomainEvent::OrganizationRenamed { .. }
        ));
        assert_eq!(o.name.as_str(), "TBZ");
    }

    #[test]
    fn identical_details_are_a_noop_without_event() {
        let mut o = org(Some(Id::new_v7()));
        let name = same_name(&o);
        let events = o.replace_details(name, None, None).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn address_change_alone_emits_no_event() {
        let mut o = org(Some(Id::new_v7()));
        let name = same_name(&o);
        let events = o.replace_details(name, Some(address()), None).unwrap();
        assert!(events.is_empty());
        assert_eq!(o.address(), Some(&address()));
    }

    #[test]
    fn address_can_be_cleared() {
        let mut o = org(Some(Id::new_v7()));
        let name = same_name(&o);
        o.replace_details(name.clone(), Some(address()), None)
            .unwrap();
        o.replace_details(name, None, None).unwrap();
        assert_eq!(o.address(), None);
    }

    #[test]
    fn contact_person_is_stored_and_cleared() {
        let mut o = org(Some(Id::new_v7()));
        let name = same_name(&o);
        let person = uuid::Uuid::now_v7();
        o.replace_details(name.clone(), None, Some(person)).unwrap();
        assert_eq!(o.contact_person(), Some(person));
        o.replace_details(name, None, None).unwrap();
        assert_eq!(o.contact_person(), None);
    }

    #[test]
    fn root_cannot_be_modified() {
        let mut root = org(None);
        assert!(root.is_root());
        let result = root.replace_details(
            OrganizationName::new("Anders").unwrap(),
            Some(address()),
            None,
        );
        assert_err!(&result);
        assert!(matches!(
            result.unwrap_err(),
            OrganizationError::RootImmutable
        ));
    }

    #[test]
    fn reconstitute_maps_snapshot_with_address() {
        let id = uuid::Uuid::now_v7();
        let parent = uuid::Uuid::now_v7();
        let person = uuid::Uuid::now_v7();
        let o = Organization::reconstitute(snapshot::OrganizationSnapshot {
            id,
            parent_id: Some(parent),
            name: "GaLaBau Müller".into(),
            street: Some("Nordergraben 12".into()),
            postal_code: Some("24937".into()),
            city: Some("Flensburg".into()),
            contact_person_id: Some(person),
        });
        assert_eq!(o.id.value(), id);
        assert_eq!(o.parent_id().unwrap().value(), parent);
        assert_ok!(OrganizationName::new(o.name.as_str()));
        assert_eq!(o.address(), Some(&address()));
        assert_eq!(o.contact_person(), Some(person));
    }

    #[test]
    fn reconstitute_maps_snapshot_without_address() {
        let o = Organization::reconstitute(snapshot::OrganizationSnapshot {
            id: uuid::Uuid::now_v7(),
            parent_id: Some(uuid::Uuid::now_v7()),
            name: "GaLaBau Müller".into(),
            street: None,
            postal_code: None,
            city: None,
            contact_person_id: None,
        });
        assert_eq!(o.address(), None);
        assert_eq!(o.contact_person(), None);
    }

    #[test]
    fn view_carries_member_count_and_address() {
        let mut o = org(Some(Id::new_v7()));
        let name = same_name(&o);
        o.replace_details(name, Some(address()), None).unwrap();
        let view = OrganizationView::new(&o, 14);
        assert_eq!(view.member_count, 14);
        assert_eq!(view.address, Some(address()));
        assert_eq!(view.name, o.name);
    }
}
