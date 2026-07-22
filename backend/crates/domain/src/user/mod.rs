//! User read model — identity and profile merged into one view.
//!
//! There is no User aggregate: identity lifecycle and invariants (username,
//! email, roles) live entirely with the IdP and are surfaced as
//! [`UserIdentity`]. Application-owned facts (contact details, status,
//! driving licenses) live separately as [`UserProfile`]. [`UserView`] is the
//! flat, merged read model the API returns; it carries `created_at` from the
//! identity side. [`UserRepository`] covers identity access only (no
//! reader/writer split, since identities have no local snapshot to
//! rehydrate); [`UserProfileReader`]/[`UserProfileWriter`] cover the profile
//! side. The `id` is a [`Uuid`] (the IdP's own identifier) rather than an
//! `Id<…>`.

pub mod profile;

pub use profile::UserProfile;

use std::str::FromStr;

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use url::Url;
use uuid::Uuid;

use crate::{
    Id, RepositoryError,
    organization::Organization,
    shared::{
        email::Email,
        error::ValidationError,
        pagination::{Page, Pagination},
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
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "user_status", rename_all = "lowercase")
)]
pub enum UserStatus {
    Available,
    Absent,
}

crate::newtype_nonempty! {
    /// Keycloak username, 1–64 characters after trimming.
    Username, "user.username", 1, 64
}

/// Flat read model for a Keycloak-managed user.
///
/// Carries `created_at` (a DB-style audit field) because there is no separate
/// User aggregate — Keycloak owns the lifecycle. Returned by every method on
/// [`UserRepository`] for both reads and the post-create response.
#[derive(Debug, Clone)]
pub struct UserView {
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
    pub status: UserStatus,
    pub driving_licenses: Vec<DrivingLicense>,
}

/// Identity facts owned by the IdP; merged with `UserProfile` into `UserView`.
#[derive(Debug, Clone)]
pub struct UserIdentity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub username: Username,
    pub first_name: String,
    pub last_name: String,
    pub email: Email,
    pub email_verified: bool,
    pub roles: Vec<UserRole>,
}

/// Unified access to IdP-managed user identities.
///
/// Not split into reader/writer because identity management is entirely
/// delegated to the IdP — there is no local snapshot to rehydrate.
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, entity: UserCreate) -> Result<UserIdentity, RepositoryError>;
    async fn all(&self, pagination: Pagination) -> Result<Page<UserIdentity>, RepositoryError>;
    async fn by_role(
        &self,
        role: UserRole,
        pagination: Pagination,
    ) -> Result<Page<UserIdentity>, RepositoryError>;
    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserIdentity>, RepositoryError>;
}

#[async_trait::async_trait]
pub trait UserProfileReader: Send + Sync {
    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError>;
    async fn ids_in_organization(
        &self,
        org: Id<Organization>,
    ) -> Result<Vec<Uuid>, RepositoryError>;
    async fn organizations_for(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, Id<Organization>)>, RepositoryError>;
}

#[async_trait::async_trait]
pub trait UserProfileWriter: Send + Sync {
    async fn upsert(&self, profile: &UserProfile) -> Result<(), RepositoryError>;
    /// Creates an empty profile row if none exists — never touches existing data.
    async fn ensure_exists(&self, id: Uuid) -> Result<(), RepositoryError>;
    async fn set_organization(
        &self,
        id: Uuid,
        org: Id<Organization>,
    ) -> Result<(), RepositoryError>;
}
