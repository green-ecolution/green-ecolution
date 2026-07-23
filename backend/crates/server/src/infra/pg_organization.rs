use sqlx::PgPool;

use domain::organization::OrganizationSnapshot;
use domain::{
    Id, RepositoryError,
    authorization::OrgHierarchy,
    organization::{Organization, OrganizationDraft, OrganizationReader, OrganizationWriter},
    role::Role,
};

pub struct PgOrganizationRepository {
    pool: PgPool,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OrganizationReader for PgOrganizationRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
        let orgs = sqlx::query_as!(
            OrganizationSnapshot,
            r#"SELECT id, parent_id, name FROM organizations ORDER BY name ASC, id ASC"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Organization::reconstitute)
        .collect();
        Ok(orgs)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<Organization>) -> Result<Organization, RepositoryError> {
        sqlx::query_as!(
            OrganizationSnapshot,
            r#"SELECT id, parent_id, name FROM organizations WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)
        .map(Organization::reconstitute)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
        let rows = sqlx::query!(r#"SELECT id, parent_id FROM organizations"#)
            .fetch_all(&self.pool)
            .await?;
        Ok(OrgHierarchy::from_pairs(rows.into_iter().map(|r| {
            (
                Id::<Organization>::new(r.id),
                r.parent_id.map(Id::<Organization>::new),
            )
        })))
    }
}

#[async_trait::async_trait]
impl OrganizationWriter for PgOrganizationRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn save_new(
        &self,
        draft: OrganizationDraft,
        templates: Vec<Role>,
    ) -> Result<Organization, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        let id = Id::<Organization>::new_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, $3)"#,
            id.value(),
            draft.parent_id.value(),
            draft.name.as_str(),
        )
        .execute(&mut *tx)
        .await?;

        for template in &templates {
            let role_draft = template.copy_for(id);
            let role_id = Id::<Role>::new_v7();
            let permissions: Vec<String> = role_draft
                .permissions
                .iter()
                .map(|p| p.to_string())
                .collect();
            sqlx::query!(
                r#"INSERT INTO roles (id, organization_id, name, description, permissions)
                   VALUES ($1, $2, $3, $4, $5)"#,
                role_id.value(),
                role_draft.organization_id.value(),
                role_draft.name.as_str(),
                role_draft.description.as_ref().map(|d| d.as_str()),
                &permissions,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(Organization::reconstitute(OrganizationSnapshot {
            id: id.value(),
            parent_id: Some(draft.parent_id.value()),
            name: draft.name.as_str().to_string(),
        }))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn save(&self, org: &Organization) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"UPDATE organizations SET name = $2 WHERE id = $1"#,
            org.id.value(),
            org.name.as_str(),
        )
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn delete(&self, id: Id<Organization>) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM organizations WHERE id = $1"#, id.value())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}
