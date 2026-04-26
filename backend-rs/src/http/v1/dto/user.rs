use serde::{Deserialize, Serialize};

use crate::domain::auth::{ClientToken, LoginResponse as DomainLoginResponse};

use super::{DrivingLicense, UserRole, UserStatus};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub created_at: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_verified: bool,
    pub employee_id: String,
    pub phone_number: String,
    pub avatar_url: String,
    pub status: UserStatus,
    pub roles: Vec<UserRole>,
    pub driving_licenses: Vec<DrivingLicense>,
}

#[derive(Debug, Deserialize)]
pub struct UserRegisterRequest {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub login_url: String,
}

impl From<&DomainLoginResponse> for LoginResponse {
    fn from(value: &DomainLoginResponse) -> Self {
        Self {
            login_url: value.login_url.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginTokenRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
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
