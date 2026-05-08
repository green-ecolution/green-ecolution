use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::service::ServiceError;
use domain::{
    auth::{ClientToken, LoginResponse as DomainLoginResponse},
    shared::email::Email,
    user::{
        UserCreate as DomainUserCreate, UserRole as DomainUserRole, UserStatus as DomainUserStatus,
        UserView as DomainUserView, Username,
    },
};

use super::{DrivingLicense, UserRole, UserStatus};

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
    "roles": ["Tbz"],
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
    /// Assigned roles.
    pub roles: Vec<UserRole>,
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
    "roles": ["Tbz"],
    "employee_id": "EMP-042",
    "phone_number": "+49 461 123456",
    "avatar_url": "https://example.com/avatar.jpg"
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
    /// Roles to assign to the user.
    pub roles: Vec<String>,
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
}

/// Response containing the OIDC login URL for browser-based authentication.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    /// Full OIDC authorization URL the client should redirect to.
    #[schema(
        example = "https://auth.green-ecolution.de/auth/realms/green-ecolution/protocol/openid-connect/auth?client_id=green-ecolution&redirect_uri=https%3A%2F%2Fapp.green-ecolution.de%2Fcallback&response_type=code&scope=openid+profile+email"
    )]
    pub login_url: String,
}

impl From<&DomainLoginResponse> for LoginResponse {
    fn from(value: &DomainLoginResponse) -> Self {
        Self {
            login_url: value.login_url.to_string(),
        }
    }
}

/// Request body for exchanging an authorization code for tokens.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginTokenRequest {
    /// Authorization code received from the OIDC provider callback.
    #[schema(example = "abc123-auth-code")]
    pub code: String,
    /// PKCE code verifier matching the `code_challenge` sent on /login. Required
    /// when the frontend client is public (no secret); omitted otherwise.
    #[serde(default)]
    #[schema(example = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk", nullable)]
    pub code_verifier: Option<String>,
}

/// Request body for logging out a user session.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LogoutRequest {
    /// Refresh token of the session to invalidate.
    #[schema(example = "eyJhbGciOiJSUzI1NiIs...")]
    pub refresh_token: String,
}

/// Request body for refreshing an access token.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token used to obtain a new access token.
    #[schema(example = "eyJhbGciOiJSUzI1NiIs...")]
    pub refresh_token: String,
}

/// OIDC token response containing access, ID, and refresh tokens.
#[derive(Debug, Serialize, utoipa::ToSchema)]
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
    /// JWT access token for API authorization.
    #[schema(example = "eyJhbGciOiJSUzI1NiIs...")]
    pub access_token: String,
    /// JWT ID token containing user identity claims.
    #[schema(example = "eyJhbGciOiJSUzI1NiIs...")]
    pub id_token: String,
    /// Absolute expiry timestamp of the access token (RFC 3339).
    #[schema(example = "2024-08-01T13:00:00+00:00")]
    pub expiry: String,
    /// Access token lifetime in seconds.
    #[schema(example = 300)]
    pub expires_in: u32,
    /// Refresh token lifetime in seconds.
    #[schema(example = 1800)]
    pub refresh_expires_in: u32,
    /// Refresh token for obtaining new access tokens.
    #[schema(example = "eyJhbGciOiJSUzI1NiIs...")]
    pub refresh_token: String,
    /// Token type, typically "Bearer".
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Not-before policy timestamp (usually 0).
    #[schema(example = 0)]
    pub not_before_policy: u32,
    /// Unique session identifier.
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub session_state: String,
    /// OAuth2 scopes granted.
    #[schema(example = "openid profile email")]
    pub scope: String,
}

impl From<DomainUserRole> for UserRole {
    fn from(value: DomainUserRole) -> Self {
        match value {
            DomainUserRole::Tbz => UserRole::Tbz,
            DomainUserRole::GreenEcolution => UserRole::GreenEcolution,
            DomainUserRole::SmarteGrenzregion => UserRole::SmarteGrenzregion,
        }
    }
}

impl From<UserRole> for DomainUserRole {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Tbz => DomainUserRole::Tbz,
            UserRole::GreenEcolution => DomainUserRole::GreenEcolution,
            UserRole::SmarteGrenzregion => DomainUserRole::SmarteGrenzregion,
        }
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
            roles: value.roles.iter().copied().map(Into::into).collect(),
            driving_licenses: value
                .driving_licenses
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<UserRegisterRequest> for DomainUserCreate {
    type Error = ServiceError;

    fn try_from(value: UserRegisterRequest) -> Result<Self, Self::Error> {
        let username = Username::new(value.username)?;
        let email = Email::new(value.email)?;
        let roles = value
            .roles
            .iter()
            .map(|s| s.parse::<DomainUserRole>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
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
            roles,
            employee_id: value.employee_id.filter(|s| !s.is_empty()),
            phone_number: value.phone_number.filter(|s| !s.is_empty()),
            avatar_url,
        })
    }
}

impl From<&ClientToken> for ClientTokenResponse {
    fn from(value: &ClientToken) -> Self {
        Self {
            access_token: value.access_token.clone(),
            id_token: value.id_token.clone(),
            expiry: value.expiry.to_rfc3339(),
            expires_in: value.expires_in,
            refresh_expires_in: value.refresh_expires_in,
            refresh_token: value.refresh_token.clone(),
            token_type: value.token_type.clone(),
            not_before_policy: value.not_before_policy,
            session_state: value.session_state.clone(),
            scope: value.scope.clone(),
        }
    }
}
