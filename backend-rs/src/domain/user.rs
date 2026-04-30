use std::str::FromStr;

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use url::Url;
use uuid::Uuid;

use crate::domain::{
    DomainError, RepositoryError,
    shared::pagination::{Page, Pagination},
    vehicle::DrivingLicense,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    Tbz,
    GreenEcolution,
    SmarteGrenzregion,
    Unknown,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Tbz => "tbz",
            UserRole::GreenEcolution => "green-ecolution",
            UserRole::SmarteGrenzregion => "smarte-grenzregion",
            UserRole::Unknown => "unknown",
        }
    }
}

impl FromStr for UserRole {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tbz" => Ok(Self::Tbz),
            "green-ecolution" => Ok(Self::GreenEcolution),
            "smarte-grenzregion" => Ok(Self::SmarteGrenzregion),
            "unknown" => Ok(Self::Unknown),
            _ => Err(DomainError::InvalidInput(format!("unknown user role: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Available,
    Absent,
    Unknown,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Available => "available",
            UserStatus::Absent => "absent",
            UserStatus::Unknown => "unknown",
        }
    }
}

impl FromStr for UserStatus {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(Self::Available),
            "absent" => Ok(Self::Absent),
            "unknown" => Ok(Self::Unknown),
            _ => Err(DomainError::InvalidInput(format!(
                "unknown user status: {s}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_verified: bool,
    pub employee_id: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_url: Option<Url>,
    pub roles: Vec<UserRole>,
    pub driving_licenses: Vec<DrivingLicense>,
    pub status: UserStatus,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: SecretString,
    pub roles: Vec<String>,
    pub employee_id: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_url: Option<Url>,
}

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, entity: UserCreate) -> Result<User, RepositoryError>;
    async fn all(&self, pagination: Pagination) -> Result<Page<User>, RepositoryError>;
    async fn by_role(
        &self,
        role: UserRole,
        pagination: Pagination,
    ) -> Result<Page<User>, RepositoryError>;
    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, RepositoryError>;
    async fn revoke_session(&self, refresh_token: &str) -> Result<(), RepositoryError>;
}
