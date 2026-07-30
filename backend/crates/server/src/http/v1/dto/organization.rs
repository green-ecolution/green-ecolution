use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::{
    organization::{OrganizationDetailView, OrganizationView},
    shared::address::{Address, City, PostalCode, Street},
};

/// A postal address. All three parts are required together.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "street": "Nordergraben 12",
    "postal_code": "24937",
    "city": "Flensburg"
}))]
pub struct AddressDto {
    pub street: String,
    pub postal_code: String,
    pub city: String,
}

impl TryFrom<AddressDto> for Address {
    type Error = domain::shared::error::ValidationError;

    fn try_from(dto: AddressDto) -> Result<Self, Self::Error> {
        Ok(Address::new(
            Street::new(dto.street)?,
            PostalCode::new(dto.postal_code)?,
            City::new(dto.city)?,
        ))
    }
}

impl From<&Address> for AddressDto {
    fn from(value: &Address) -> Self {
        Self {
            street: value.street().as_str().to_string(),
            postal_code: value.postal_code().as_str().to_string(),
            city: value.city().as_str().to_string(),
        }
    }
}

/// The organization's contact person, resolved from the identity provider.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ContactPersonDto {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

/// A node in the organization tree.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "01980000-0000-7000-8000-000000000001",
    "parent_id": null,
    "name": "Green Ecolution",
    "address": null,
    "contact_person_id": null,
    "member_count": 14,
    "created_at": "2024-06-15T10:30:00+00:00"
}))]
pub struct OrganizationResponse {
    /// Unique organization identifier (UUID v7).
    pub id: String,
    /// Parent organization; null only for the single root.
    pub parent_id: Option<String>,
    /// Display name, unique among siblings.
    pub name: String,
    /// Postal address; null when the organization has none.
    pub address: Option<AddressDto>,
    /// Contact person id; resolve via the detail endpoint for name and email.
    pub contact_person_id: Option<String>,
    /// Directly assigned users, excluding sub-organizations.
    pub member_count: i64,
    /// Creation time derived from the UUID v7 id (null for seeded ids without a real timestamp).
    pub created_at: Option<String>,
}

impl From<&OrganizationView> for OrganizationResponse {
    fn from(value: &OrganizationView) -> Self {
        Self {
            id: value.id.to_string(),
            parent_id: value.parent_id.map(|p| p.to_string()),
            name: value.name.as_str().to_string(),
            address: value.address.as_ref().map(AddressDto::from),
            contact_person_id: value.contact_person_id.map(|p| p.to_string()),
            member_count: value.member_count,
            created_at: value.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// A single organization with its contact person resolved.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationDetailResponse {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub address: Option<AddressDto>,
    /// The stored reference, present even when the identity provider cannot
    /// resolve it. Clients must round-trip this on update instead of deriving
    /// it from `contact_person`, which would silently clear the reference.
    pub contact_person_id: Option<String>,
    /// Null when no contact person is stored *or* when the stored id could not
    /// be resolved; `contact_person_id` distinguishes the two.
    pub contact_person: Option<ContactPersonDto>,
    pub member_count: i64,
    pub created_at: Option<String>,
}

impl From<&OrganizationDetailView> for OrganizationDetailResponse {
    fn from(value: &OrganizationDetailView) -> Self {
        let org = &value.organization;
        Self {
            id: org.id.to_string(),
            parent_id: org.parent_id.map(|p| p.to_string()),
            name: org.name.as_str().to_string(),
            address: org.address.as_ref().map(AddressDto::from),
            contact_person_id: org.contact_person_id.map(|p| p.to_string()),
            contact_person: value.contact_person.as_ref().map(|p| ContactPersonDto {
                id: p.id.to_string(),
                first_name: p.first_name.clone(),
                last_name: p.last_name.clone(),
                email: p.email.as_str().to_string(),
            }),
            member_count: org.member_count,
            created_at: org.created_at.map(|t| t.to_rfc3339()),
        }
    }
}

/// Request body for creating an organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "name": "TBZ Flensburg", "parent_id": "01980000-0000-7000-8000-000000000001" }))]
pub struct OrganizationCreateRequest {
    pub name: String,
    pub parent_id: Uuid,
}

/// Request body for updating an organization. Replaces the whole editable set —
/// an omitted field clears the stored value.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "name": "Stadtgärtnerei Nord",
    "address": { "street": "Nordergraben 12", "postal_code": "24937", "city": "Flensburg" },
    "contact_person_id": null
}))]
pub struct OrganizationUpdateRequest {
    pub name: String,
    #[serde(default)]
    pub address: Option<AddressDto>,
    #[serde(default)]
    pub contact_person_id: Option<Uuid>,
}
