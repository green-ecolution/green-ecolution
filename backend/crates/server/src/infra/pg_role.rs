use sqlx::PgPool;
use uuid::Uuid;

use domain::role::RoleSnapshot;
use domain::{
    Id, RepositoryError,
    organization::Organization,
    role::{Role, RoleDraft, RoleReader, RoleWriter},
};

pub struct PgRoleRepository {
    pool: PgPool,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn permissions_as_strings(
    role_or_draft_permissions: impl IntoIterator<Item = domain::authorization::Permission>,
) -> Vec<String> {
    role_or_draft_permissions
        .into_iter()
        .map(|p| p.to_string())
        .collect()
}

#[async_trait::async_trait]
impl RoleReader for PgRoleRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Role>) -> Result<Role, RepositoryError> {
        let snap = sqlx::query_as!(
            RoleSnapshot,
            r#"SELECT id, organization_id, name, description, permissions FROM roles WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;
        Ok(Role::reconstitute(snap)?)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_organization(&self, org: Id<Organization>) -> Result<Vec<Role>, RepositoryError> {
        sqlx::query_as!(
            RoleSnapshot,
            r#"SELECT id, organization_id, name, description, permissions
               FROM roles WHERE organization_id = $1 ORDER BY name ASC"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|s| Role::reconstitute(s).map_err(Into::into))
        .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
        sqlx::query_as!(
            RoleSnapshot,
            r#"SELECT id, organization_id, name, description, permissions
               FROM roles WHERE organization_id IS NULL ORDER BY name ASC"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|s| Role::reconstitute(s).map_err(Into::into))
        .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn roles_for_user(&self, user_id: Uuid) -> Result<Vec<Role>, RepositoryError> {
        sqlx::query_as!(
            RoleSnapshot,
            r#"SELECT r.id, r.organization_id, r.name, r.description, r.permissions
               FROM roles r JOIN role_assignments a ON a.role_id = r.id
               WHERE a.user_id = $1"#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|s| Role::reconstitute(s).map_err(Into::into))
        .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn roles_for_users(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, Role)>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT a.user_id, r.id, r.organization_id, r.name, r.description, r.permissions
               FROM roles r JOIN role_assignments a ON a.role_id = r.id
               WHERE a.user_id = ANY($1)"#,
            ids
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                let role = Role::reconstitute(RoleSnapshot {
                    id: row.id,
                    organization_id: row.organization_id,
                    name: row.name,
                    description: row.description,
                    permissions: row.permissions,
                })?;
                Ok((row.user_id, role))
            })
            .collect()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn user_ids_with_role(&self, role_id: Id<Role>) -> Result<Vec<Uuid>, RepositoryError> {
        let ids = sqlx::query_scalar!(
            r#"SELECT user_id FROM role_assignments WHERE role_id = $1"#,
            role_id.value()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }
}

#[async_trait::async_trait]
impl RoleWriter for PgRoleRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(&self, draft: RoleDraft) -> Result<Role, RepositoryError> {
        let id = Id::<Role>::new_v7();
        let permissions = permissions_as_strings(draft.permissions.iter().copied());
        sqlx::query!(
            r#"INSERT INTO roles (id, organization_id, name, description, permissions)
               VALUES ($1, $2, $3, $4, $5)"#,
            id.value(),
            draft.organization_id.value(),
            draft.name.as_str(),
            draft.description.as_ref().map(|d| d.as_str()),
            &permissions,
        )
        .execute(&self.pool)
        .await?;
        Ok(Role::reconstitute(RoleSnapshot {
            id: id.value(),
            organization_id: Some(draft.organization_id.value()),
            name: draft.name.as_str().to_string(),
            description: draft.description.as_ref().map(|d| d.as_str().to_string()),
            permissions,
        })?)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, role: &Role) -> Result<(), RepositoryError> {
        let permissions = permissions_as_strings(role.permissions().iter().copied());
        let result = sqlx::query!(
            r#"UPDATE roles SET name = $2, description = $3, permissions = $4 WHERE id = $1"#,
            role.id.value(),
            role.name.as_str(),
            role.description.as_ref().map(|d| d.as_str()),
            &permissions,
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Role>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM roles WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn assign_to_user(
        &self,
        user_id: Uuid,
        role_id: Id<Role>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"INSERT INTO role_assignments (user_id, role_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING"#,
            user_id,
            role_id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn revoke_from_user(
        &self,
        user_id: Uuid,
        role_id: Id<Role>,
    ) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"DELETE FROM role_assignments WHERE user_id = $1 AND role_id = $2"#,
            user_id,
            role_id.value()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
