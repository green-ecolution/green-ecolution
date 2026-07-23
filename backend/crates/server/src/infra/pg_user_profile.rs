use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    organization::Organization,
    user::{UserProfile, UserProfileReader, UserProfileWriter, UserStatus},
    vehicle::DrivingLicense,
};

pub struct PgUserProfileRepository {
    pool: PgPool,
}

impl PgUserProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct UserProfileRow {
    id: Uuid,
    employee_id: Option<String>,
    phone_number: Option<String>,
    avatar_url: Option<String>,
    status: UserStatus,
    driving_licenses: Vec<DrivingLicense>,
}

impl UserProfileRow {
    fn try_into_domain(self) -> Result<UserProfile, RepositoryError> {
        let avatar_url = self
            .avatar_url
            .as_deref()
            .map(Url::parse)
            .transpose()
            .map_err(|e| RepositoryError::DataIntegrity(format!("invalid avatar_url: {e}")))?;
        Ok(UserProfile {
            id: self.id,
            employee_id: self.employee_id,
            phone_number: self.phone_number,
            avatar_url,
            status: self.status,
            driving_licenses: self.driving_licenses,
        })
    }
}

#[async_trait::async_trait]
impl UserProfileReader for PgUserProfileRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_ids(&self, ids: &[Uuid]) -> Result<Vec<UserProfile>, RepositoryError> {
        sqlx::query_as!(
            UserProfileRow,
            r#"SELECT id, employee_id, phone_number, avatar_url,
                      status AS "status: UserStatus",
                      driving_licenses AS "driving_licenses: Vec<DrivingLicense>"
               FROM user_profiles
               WHERE id = ANY($1)"#,
            ids
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(UserProfileRow::try_into_domain)
        .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn ids_in_organization(
        &self,
        org: Id<Organization>,
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let ids = sqlx::query_scalar!(
            r#"SELECT id FROM user_profiles WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn organizations_for(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, Id<Organization>)>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT id, organization_id AS "organization_id!" FROM user_profiles
               WHERE id = ANY($1) AND organization_id IS NOT NULL"#,
            ids
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.id, Id::new(r.organization_id)))
            .collect())
    }
}

#[async_trait::async_trait]
impl UserProfileWriter for PgUserProfileRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn upsert(&self, profile: &UserProfile) -> Result<(), RepositoryError> {
        let avatar_url = profile.avatar_url.as_ref().map(Url::to_string);
        sqlx::query!(
            r#"INSERT INTO user_profiles
                   (id, employee_id, phone_number, avatar_url, status, driving_licenses)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (id) DO UPDATE SET
                   employee_id      = EXCLUDED.employee_id,
                   phone_number     = EXCLUDED.phone_number,
                   avatar_url       = EXCLUDED.avatar_url,
                   status           = EXCLUDED.status,
                   driving_licenses = EXCLUDED.driving_licenses"#,
            profile.id,
            profile.employee_id.as_deref(),
            profile.phone_number.as_deref(),
            avatar_url.as_deref(),
            profile.status as UserStatus,
            profile.driving_licenses.as_slice() as &[DrivingLicense],
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn ensure_exists(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO user_profiles (id) VALUES ($1) ON CONFLICT (id) DO NOTHING"#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn set_organization(
        &self,
        id: Uuid,
        org: Id<Organization>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO user_profiles (id, organization_id) VALUES ($1, $2)
               ON CONFLICT (id) DO UPDATE SET organization_id = EXCLUDED.organization_id"#,
            id,
            org.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
