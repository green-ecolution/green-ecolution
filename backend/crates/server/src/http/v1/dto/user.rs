use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::ServiceError;
use domain::{
    Id,
    organization::Organization,
    role::Role,
    shared::email::Email,
    user::{
        UserCreate as DomainUserCreate, UserProfile as DomainUserProfile,
        UserStatus as DomainUserStatus, UserView as DomainUserView, Username,
    },
};

use super::{DrivingLicense, UserStatus, organization::OrganizationResponse, role::RoleResponse};
use crate::http::v1::pagination::PaginationParams;

/// Represents a user account in the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "created_at": "2024-06-15T10:30:00+00:00",
    "username": "jdoe",
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@tbz-flensburg.de",
    "email_verified": true,
    "employee_id": "EMP-042",
    "phone_number": "+49 461 123456",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "Available",
    "organization": null,
    "roles": [],
    "driving_licenses": ["B"]
}))]
pub struct UserResponse {
    /// Unique user identifier (UUID).
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Timestamp when the user was created.
    #[schema(example = "2024-06-15T10:30:00+00:00")]
    pub created_at: String,
    /// Login username.
    #[schema(example = "jdoe")]
    pub username: String,
    /// User's first name.
    #[schema(example = "John")]
    pub first_name: String,
    /// User's last name.
    #[schema(example = "Doe")]
    pub last_name: String,
    /// Email address.
    #[schema(example = "john.doe@tbz-flensburg.de")]
    pub email: String,
    /// Whether the email has been verified.
    #[schema(example = true)]
    pub email_verified: bool,
    /// Internal employee identifier.
    #[schema(example = "EMP-042")]
    pub employee_id: String,
    /// Contact phone number.
    #[schema(example = "+49 461 123456")]
    pub phone_number: String,
    /// URL to the user's avatar image.
    #[schema(example = "https://example.com/avatar.jpg")]
    pub avatar_url: String,
    /// Current availability status.
    pub status: UserStatus,
    /// Organization the user belongs to; null for legacy users without one.
    pub organization: Option<OrganizationResponse>,
    /// Roles assigned to the user (org-scoped permission sets).
    pub roles: Vec<RoleResponse>,
    /// Driving licenses held by the user.
    pub driving_licenses: Vec<DrivingLicense>,
}

/// Request body for registering a new user.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "username": "jdoe",
    "first_name": "John",
    "last_name": "Doe",
    "email": "john.doe@tbz-flensburg.de",
    "password": "s3cur3P@ss!",
    "organization_id": "01980000-0000-7000-8000-000000000001",
    "role_ids": [],
    "employee_id": "EMP-042",
    "phone_number": "+49 461 123456",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "available",
    "driving_licenses": ["B"]
}))]
pub struct UserRegisterRequest {
    /// Desired login username.
    #[schema(example = "jdoe")]
    pub username: String,
    /// User's first name.
    #[schema(example = "John")]
    pub first_name: String,
    /// User's last name.
    #[schema(example = "Doe")]
    pub last_name: String,
    /// Email address.
    #[schema(example = "john.doe@tbz-flensburg.de")]
    pub email: String,
    /// Password for the new account.
    #[schema(example = "s3cur3P@ss!")]
    pub password: String,
    /// Organization the new user belongs to.
    pub organization_id: Uuid,
    /// Roles to assign to the user (must be org-owned, not templates).
    #[serde(default)]
    pub role_ids: Vec<Uuid>,
    /// Optional internal employee identifier.
    #[serde(default)]
    #[schema(example = "EMP-042", nullable)]
    pub employee_id: Option<String>,
    /// Optional contact phone number.
    #[serde(default)]
    #[schema(example = "+49 461 123456", nullable)]
    pub phone_number: Option<String>,
    /// Optional URL to the user's avatar image.
    #[serde(default)]
    #[schema(example = "https://example.com/avatar.jpg", nullable)]
    pub avatar_url: Option<String>,
    /// Initial availability status (defaults to Available).
    #[serde(default)]
    #[schema(nullable)]
    pub status: Option<UserStatus>,
    /// Initial driving licenses.
    #[serde(default)]
    pub driving_licenses: Vec<DrivingLicense>,
}

/// Request body for replacing a user's profile data.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "employee_id": "EMP-042",
    "phone_number": "+49 461 123456",
    "avatar_url": "https://example.com/avatar.jpg",
    "status": "available",
    "driving_licenses": ["B", "CE"]
}))]
pub struct UserUpdateRequest {
    /// Internal employee identifier; empty or absent clears the value.
    #[serde(default)]
    #[schema(example = "EMP-042", nullable)]
    pub employee_id: Option<String>,
    /// Contact phone number; empty or absent clears the value.
    #[serde(default)]
    #[schema(example = "+49 461 123456", nullable)]
    pub phone_number: Option<String>,
    /// Avatar URL; empty or absent clears the value.
    #[serde(default)]
    #[schema(example = "https://example.com/avatar.jpg", nullable)]
    pub avatar_url: Option<String>,
    /// New availability status.
    pub status: UserStatus,
    /// Full replacement set of driving licenses.
    pub driving_licenses: Vec<DrivingLicense>,
}

/// Query parameters for the user list endpoint: pagination plus optional
/// organization/role filters resolved against the local database.
#[derive(Debug, Deserialize)]
pub struct UserListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    #[serde(default)]
    pub organization_id: Option<Uuid>,
    #[serde(default)]
    pub role_id: Option<Uuid>,
}

/// Request body for assigning a role to a user.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "role_id": "01980000-0000-7000-8000-0000000000b2" }))]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

/// Request body for changing a user's organization.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "organization_id": "01980000-0000-7000-8000-000000000001" }))]
pub struct SetOrganizationRequest {
    pub organization_id: Uuid,
}

impl From<UserStatus> for DomainUserStatus {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Available => Self::Available,
            UserStatus::Absent => Self::Absent,
        }
    }
}

impl UserUpdateRequest {
    pub fn try_into_profile(self, id: uuid::Uuid) -> Result<DomainUserProfile, ServiceError> {
        let avatar_url = self
            .avatar_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(url::Url::parse)
            .transpose()
            .map_err(|e| ServiceError::InvalidInput(format!("avatar_url: {e}")))?;
        Ok(DomainUserProfile {
            id,
            employee_id: self.employee_id.filter(|s| !s.is_empty()),
            phone_number: self.phone_number.filter(|s| !s.is_empty()),
            avatar_url,
            status: self.status.into(),
            driving_licenses: self.driving_licenses.into_iter().map(Into::into).collect(),
        })
    }
}

impl From<DomainUserStatus> for UserStatus {
    fn from(value: DomainUserStatus) -> Self {
        match value {
            DomainUserStatus::Available => UserStatus::Available,
            DomainUserStatus::Absent => UserStatus::Absent,
        }
    }
}

impl From<&DomainUserView> for UserResponse {
    fn from(value: &DomainUserView) -> Self {
        Self {
            id: value.id.to_string(),
            created_at: value.created_at.to_rfc3339(),
            username: value.username.as_str().to_string(),
            first_name: value.first_name.clone(),
            last_name: value.last_name.clone(),
            email: value.email.as_str().to_string(),
            email_verified: value.email_verified,
            employee_id: value.employee_id.clone().unwrap_or_default(),
            phone_number: value.phone_number.clone().unwrap_or_default(),
            avatar_url: value
                .avatar_url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            status: value.status.into(),
            organization: value.organization.as_ref().map(OrganizationResponse::from),
            roles: value.roles.iter().map(RoleResponse::from).collect(),
            driving_licenses: value
                .driving_licenses
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

/// OIDC token response containing access, ID, and refresh tokens.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "access_token": "eyJhbGciOiJSUzI1NiIs...",
    "id_token": "eyJhbGciOiJSUzI1NiIs...",
    "expiry": "2024-08-01T13:00:00+00:00",
    "expires_in": 300,
    "refresh_expires_in": 1800,
    "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
    "token_type": "Bearer",
    "not_before_policy": 0,
    "session_state": "550e8400-e29b-41d4-a716-446655440000",
    "scope": "openid profile email"
}))]
pub struct ClientTokenResponse {
    pub access_token: String,
    pub id_token: String,
    pub expiry: String,
    pub expires_in: u32,
    pub refresh_expires_in: u32,
    pub refresh_token: String,
    pub token_type: String,
    pub not_before_policy: u32,
    pub session_state: String,
    pub scope: String,
}

impl TryFrom<UserRegisterRequest> for DomainUserCreate {
    type Error = ServiceError;

    fn try_from(value: UserRegisterRequest) -> Result<Self, Self::Error> {
        let username = Username::new(value.username)?;
        let email = Email::new(value.email)?;
        let role_ids: Vec<Id<Role>> = value.role_ids.into_iter().map(Id::new).collect();
        let avatar_url = value
            .avatar_url
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(url::Url::parse)
            .transpose()
            .map_err(|e| ServiceError::InvalidInput(format!("avatar_url: {e}")))?;

        Ok(Self {
            username,
            first_name: value.first_name,
            last_name: value.last_name,
            email,
            password: SecretString::from(value.password),
            organization_id: Id::<Organization>::new(value.organization_id),
            role_ids,
            employee_id: value.employee_id.filter(|s| !s.is_empty()),
            phone_number: value.phone_number.filter(|s| !s.is_empty()),
            avatar_url,
            status: value
                .status
                .map(DomainUserStatus::from)
                .unwrap_or(DomainUserStatus::Available),
            driving_licenses: value.driving_licenses.into_iter().map(Into::into).collect(),
        })
    }
}
