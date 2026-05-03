//! User aggregate — a Keycloak-managed identity surfaced to the domain layer.
//!
//! Unlike other aggregates the user `id` is a [`Uuid`] (Keycloak's own identifier)
//! rather than an `Id<User>`. There is no reader/writer split: [`UserRepository`]
//! is a single unified trait because user lifecycle management is delegated
//! entirely to Keycloak and does not follow the same DB-snapshot pattern.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use url::Url;
use uuid::Uuid;

use crate::domain::{
    RepositoryError,
    shared::{
        email::Email,
        error::ValidationError,
        pagination::{Page, Pagination},
        string_value::NonEmptyString,
    },
    vehicle::DrivingLicense,
};

/// Application role assigned to a user in Keycloak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserRole {
    Tbz,
    GreenEcolution,
    SmarteGrenzregion,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Tbz => "tbz",
            UserRole::GreenEcolution => "green-ecolution",
            UserRole::SmarteGrenzregion => "smarte-grenzregion",
        }
    }
}

impl FromStr for UserRole {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tbz" => Ok(Self::Tbz),
            "green-ecolution" => Ok(Self::GreenEcolution),
            "smarte-grenzregion" => Ok(Self::SmarteGrenzregion),
            other => Err(ValidationError::InvalidFormat {
                field: "user.role",
                reason: format!("unknown role '{other}'"),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Available,
    Absent,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Available => "available",
            UserStatus::Absent => "absent",
        }
    }
}

impl FromStr for UserStatus {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(Self::Available),
            "absent" => Ok(Self::Absent),
            other => Err(ValidationError::InvalidFormat {
                field: "user.status",
                reason: format!("unknown status '{other}'"),
            }),
        }
    }
}

/// Keycloak username, 1–64 characters after trimming.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(NonEmptyString);

impl Username {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self(NonEmptyString::new(value, "user.username", 1, 64)?))
    }

    pub(crate) fn reconstitute(value: String) -> Self {
        Self(NonEmptyString::reconstitute(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub username: Username,
    pub first_name: String,
    pub last_name: String,
    pub email: Email,
    pub email_verified: bool,
    pub employee_id: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_url: Option<Url>,
    pub roles: Vec<UserRole>,
    pub driving_licenses: Vec<DrivingLicense>,
    pub status: UserStatus,
}

/// Input for creating a new user in Keycloak.
#[derive(Debug, Clone)]
pub struct UserCreate {
    pub username: Username,
    pub first_name: String,
    pub last_name: String,
    pub email: Email,
    pub password: SecretString,
    pub roles: Vec<UserRole>,
    pub employee_id: Option<String>,
    pub phone_number: Option<String>,
    pub avatar_url: Option<Url>,
}

/// Unified access to Keycloak-managed users.
///
/// Not split into reader/writer because user management is entirely delegated
/// to Keycloak — there is no local DB snapshot to rehydrate.
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
