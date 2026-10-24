//! Role aggregate — a named permission set owned by exactly one organization.
//!
//! A role grants its permissions for the owning organization plus its whole
//! subtree — never upwards. `organization_id = NULL` marks a template: it is
//! delivered by migration, immutable through the API and not assignable;
//! creating an organization instantiates org-owned copies of every template.

pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use crate::{
    Id,
    authorization::{Permission, PermissionParseError, ResourceCatalog},
    events::DomainEvent,
    organization::Organization,
    shared::error::ValidationError,
};
use std::collections::BTreeSet;

pub use error::RoleError;
pub use repository::{RoleReader, RoleWriter};
#[doc(hidden)]
pub use snapshot::RoleSnapshot;
pub use view::RoleView;

crate::newtype_nonempty! {
    /// Role display name, 1–120 characters after trimming.
    RoleName, "role.name", 1, 120
}

crate::newtype_nonempty! {
    /// Optional role description, 1–500 characters after trimming.
    RoleDescription, "role.description", 1, 500
}

crate::newtype_nonempty! {
    /// Identifies which delivered template a role descends from, e.g.
    /// `tree_care`. Present only while name and description are still the ones
    /// that shipped, which is what lets a client translate them.
    RoleTemplateKey, "role.template_key", 1, 64
}

#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub id: Id<Role>,
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    organization_id: Option<Id<Organization>>,
    permissions: BTreeSet<Permission>,
    /// Cleared as soon as name or description is edited, so a client only ever
    /// translates text the user has not made their own.
    template_key: Option<RoleTemplateKey>,
}

/// Input for creating an org-owned role (manually or as a template copy).
#[derive(Debug, Clone)]
pub struct RoleDraft {
    pub organization_id: Id<Organization>,
    pub name: RoleName,
    pub description: Option<RoleDescription>,
    pub permissions: BTreeSet<Permission>,
    pub template_key: Option<RoleTemplateKey>,
}

pub struct Reconstituted {
    pub role: Role,
    /// Stored permissions whose resource no registered module declares.
    pub ignored_permissions: Vec<String>,
}

impl Role {
    #[doc(hidden)]
    pub fn reconstitute(
        snap: RoleSnapshot,
        catalog: &ResourceCatalog,
    ) -> Result<Reconstituted, ValidationError> {
        let mut permissions = BTreeSet::new();
        let mut ignored_permissions = Vec::new();
        for raw in &snap.permissions {
            match catalog.parse_permission(raw) {
                Ok(p) => {
                    permissions.insert(p);
                }
                Err(PermissionParseError::UnknownResource(_)) => {
                    ignored_permissions.push(raw.clone());
                }
                Err(PermissionParseError::Malformed(e)) => return Err(e),
            }
        }
        let role = Self {
            id: Id::new(snap.id),
            name: RoleName::reconstitute(snap.name),
            description: snap.description.map(RoleDescription::reconstitute),
            organization_id: snap.organization_id.map(Id::new),
            permissions,
            template_key: snap.template_key.map(RoleTemplateKey::reconstitute),
        };
        Ok(Reconstituted {
            role,
            ignored_permissions,
        })
    }

    pub fn organization_id(&self) -> Option<Id<Organization>> {
        self.organization_id
    }

    pub fn permissions(&self) -> &BTreeSet<Permission> {
        &self.permissions
    }

    pub fn is_template(&self) -> bool {
        self.organization_id.is_none()
    }

    /// Which delivered template this role still matches verbatim, if any.
    pub fn template_key(&self) -> Option<&RoleTemplateKey> {
        self.template_key.as_ref()
    }

    pub fn rename(&mut self, new_name: RoleName) -> Result<Vec<DomainEvent>, RoleError> {
        self.ensure_mutable()?;
        if self.name == new_name {
            return Ok(Vec::new());
        }
        self.name = new_name;
        self.template_key = None;
        Ok(vec![DomainEvent::RoleRenamed { role_id: self.id }])
    }

    pub fn set_description(
        &mut self,
        description: Option<RoleDescription>,
    ) -> Result<(), RoleError> {
        self.ensure_mutable()?;
        self.description = description;
        self.template_key = None;
        Ok(())
    }

    pub fn replace_permissions(
        &mut self,
        permissions: BTreeSet<Permission>,
    ) -> Result<Vec<DomainEvent>, RoleError> {
        self.ensure_mutable()?;
        if self.permissions == permissions {
            return Ok(Vec::new());
        }
        self.permissions = permissions;
        Ok(vec![DomainEvent::RolePermissionsChanged {
            role_id: self.id,
        }])
    }

    /// Draft for an org-owned copy of this role (template or regular role).
    pub fn copy_for(&self, organization_id: Id<Organization>) -> RoleDraft {
        RoleDraft {
            organization_id,
            name: self.name.clone(),
            description: self.description.clone(),
            permissions: self.permissions.clone(),
            template_key: self.template_key.clone(),
        }
    }

    fn ensure_mutable(&self) -> Result<(), RoleError> {
        if self.is_template() {
            return Err(RoleError::TemplateImmutable);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{Action, Permission, ResourceKey};
    use claims::assert_err;
    use std::collections::BTreeSet;

    fn perms() -> BTreeSet<Permission> {
        BTreeSet::from([Permission::new(ResourceKey::SENSOR, Action::Read)])
    }

    fn role(org: Option<Id<Organization>>) -> Role {
        Role {
            id: Id::new_v7(),
            name: RoleName::new("Baumpflege").unwrap(),
            description: None,
            organization_id: org,
            permissions: perms(),
            template_key: None,
        }
    }

    #[test]
    fn template_has_no_organization() {
        assert!(role(None).is_template());
        assert!(!role(Some(Id::new_v7())).is_template());
    }

    #[test]
    fn template_cannot_be_renamed() {
        let mut t = role(None);
        let result = t.rename(RoleName::new("Anders").unwrap());
        assert!(matches!(result.unwrap_err(), RoleError::TemplateImmutable));
    }

    #[test]
    fn template_permissions_cannot_change() {
        let mut t = role(None);
        assert_err!(t.replace_permissions(BTreeSet::new()));
    }

    #[test]
    fn rename_emits_event_and_noop_stays_silent() {
        let mut r = role(Some(Id::new_v7()));
        assert!(
            r.rename(RoleName::new("Baumpflege").unwrap())
                .unwrap()
                .is_empty()
        );
        let events = r.rename(RoleName::new("Pflege Nord").unwrap()).unwrap();
        assert!(matches!(
            events[0],
            crate::events::DomainEvent::RoleRenamed { .. }
        ));
    }

    #[test]
    fn replace_permissions_emits_event_and_allows_empty_set() {
        let mut r = role(Some(Id::new_v7()));
        assert!(r.replace_permissions(perms()).unwrap().is_empty());
        let events = r.replace_permissions(BTreeSet::new()).unwrap();
        assert!(matches!(
            events[0],
            crate::events::DomainEvent::RolePermissionsChanged { .. }
        ));
        assert!(r.permissions().is_empty());
    }

    #[test]
    fn copy_for_binds_the_copy_to_the_target_org() {
        let target = Id::new_v7();
        let draft = role(None).copy_for(target);
        assert_eq!(draft.organization_id, target);
        assert_eq!(draft.name.as_str(), "Baumpflege");
        assert_eq!(draft.permissions, perms());
    }

    fn template_with_key() -> Role {
        let mut t = role(None);
        t.template_key = Some(RoleTemplateKey::new("tree_care").unwrap());
        t
    }

    #[test]
    fn a_copy_inherits_the_template_key() {
        let draft = template_with_key().copy_for(Id::new_v7());
        assert_eq!(
            draft.template_key.as_ref().map(RoleTemplateKey::as_str),
            Some("tree_care"),
            "the copy is still the delivered role, so a client may translate its name"
        );
    }

    #[test]
    fn renaming_a_copy_drops_the_template_key() {
        let mut r = template_with_key();
        r.organization_id = Some(Id::new_v7());

        r.rename(RoleName::new("Pflege Nord").unwrap()).unwrap();

        assert_eq!(
            r.template_key, None,
            "a hand-picked name must win over the catalog"
        );
    }

    #[test]
    fn a_rename_that_changes_nothing_keeps_the_template_key() {
        let mut r = template_with_key();
        r.organization_id = Some(Id::new_v7());

        r.rename(RoleName::new("Baumpflege").unwrap()).unwrap();

        assert!(r.template_key.is_some());
    }

    #[test]
    fn editing_the_description_drops_the_template_key() {
        let mut r = template_with_key();
        r.organization_id = Some(Id::new_v7());

        r.set_description(Some(RoleDescription::new("Eigener Text").unwrap()))
            .unwrap();

        assert_eq!(r.template_key, None);
    }

    #[test]
    fn a_hand_made_role_never_gains_a_template_key() {
        let mut r = role(Some(Id::new_v7()));
        r.rename(RoleName::new("Eigene Rolle").unwrap()).unwrap();
        assert_eq!(r.template_key, None);
    }

    #[test]
    fn reconstitute_ignores_permissions_of_unknown_resources() {
        let mut snap = snapshot::RoleSnapshot {
            id: uuid::Uuid::now_v7(),
            organization_id: None,
            name: "Kaputt".into(),
            description: None,
            permissions: vec!["sensor:read".into(), "ghost:read".into()],
            template_key: None,
        };
        let r = Role::reconstitute(snap.clone(), &ResourceCatalog::core()).unwrap();
        assert_eq!(r.ignored_permissions, vec!["ghost:read".to_string()]);
        assert!(
            r.role
                .permissions()
                .contains(&Permission::new(ResourceKey::SENSOR, Action::Read))
        );
        snap.permissions = vec!["sensor".into()];
        assert!(Role::reconstitute(snap, &ResourceCatalog::core()).is_err());
    }
}
