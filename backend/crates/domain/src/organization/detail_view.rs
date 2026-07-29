use uuid::Uuid;

use crate::shared::email::Email;

use super::OrganizationView;

/// The organization's contact person, resolved from the identity provider so
/// the detail response is self-contained.
#[derive(Debug, Clone)]
pub struct ContactPersonView {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Email,
}

/// Read model for a single organization, including the resolved contact person.
#[derive(Debug, Clone)]
pub struct OrganizationDetailView {
    pub organization: OrganizationView,
    pub contact_person: Option<ContactPersonView>,
}
