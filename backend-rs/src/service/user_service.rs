use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    shared::{
        email::Email,
        pagination::{Page, Pagination},
    },
    user::{User, UserCreate, UserRepository, UserRole, UserStatus, Username},
    vehicle::DrivingLicense,
};

use super::ServiceError;

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    enabled: bool,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>, enabled: bool) -> Self {
        Self { user_repo, enabled }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn register(&self, entity: UserCreate) -> Result<User, ServiceError> {
        if !self.enabled {
            return Ok(synthesize_registered_user(entity));
        }
        Ok(self.user_repo.create(entity).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self, pagination: Pagination) -> Result<Page<User>, ServiceError> {
        if !self.enabled {
            return Ok(demo_user_page(pagination));
        }
        Ok(self.user_repo.all(pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_role(
        &self,
        role: UserRole,
        pagination: Pagination,
    ) -> Result<Page<User>, ServiceError> {
        if !self.enabled {
            if role == UserRole::GreenEcolution {
                return Ok(demo_user_page(pagination));
            }
            return Ok(Page {
                items: Vec::new(),
                total: 0,
            });
        }
        Ok(self.user_repo.by_role(role, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, ServiceError> {
        if !self.enabled {
            let demo = demo_user();
            return Ok(ids
                .iter()
                .filter(|id| **id == demo.id)
                .map(|_| demo.clone())
                .collect());
        }
        Ok(self.user_repo.by_ids(ids).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn revoke_session(&self, refresh_token: &str) -> Result<(), ServiceError> {
        if !self.enabled {
            return Ok(());
        }
        Ok(self.user_repo.revoke_session(refresh_token).await?)
    }
}

// Must stay in lockstep with `auth_service::dummy_token` and the middleware
// bypass identity — login claims, AuthUser, and user-list entry agree.
fn demo_user() -> User {
    User {
        id: Uuid::nil(),
        created_at: Utc::now(),
        username: Username::reconstitute("ttester".to_string()),
        first_name: "Toni".into(),
        last_name: "Tester".into(),
        email: Email::reconstitute("toni.tester@green-ecolution.de".to_string()),
        email_verified: true,
        employee_id: None,
        phone_number: None,
        avatar_url: None,
        roles: vec![UserRole::GreenEcolution],
        driving_licenses: vec![
            DrivingLicense::B,
            DrivingLicense::BE,
            DrivingLicense::C,
            DrivingLicense::CE,
        ],
        status: UserStatus::Available,
    }
}

fn demo_user_page(pagination: Pagination) -> Page<User> {
    let items = if pagination.page() == 1 {
        vec![demo_user()]
    } else {
        Vec::new()
    };
    Page { items, total: 1 }
}

fn synthesize_registered_user(entity: UserCreate) -> User {
    let roles = if entity.roles.is_empty() {
        vec![UserRole::GreenEcolution]
    } else {
        entity.roles.clone()
    };
    User {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        username: entity.username,
        first_name: entity.first_name,
        last_name: entity.last_name,
        email: entity.email,
        email_verified: false,
        employee_id: entity.employee_id,
        phone_number: entity.phone_number,
        avatar_url: entity.avatar_url,
        roles,
        driving_licenses: Vec::new(),
        status: UserStatus::Available,
    }
}
