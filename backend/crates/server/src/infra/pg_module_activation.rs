use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use async_trait::async_trait;
use domain::{
    Id, RepositoryError,
    authorization::OrgHierarchy,
    module::{
        ModuleActivationReader, ModuleActivationWriter, ModuleKey, ModuleState, SwitchHolder,
        module_state,
    },
    organization::{Organization, OrganizationReader},
};
use sqlx::PgPool;

pub struct PgModuleActivationRepository {
    pool: PgPool,
    org_reader: Arc<dyn OrganizationReader>,
    defaults: BTreeMap<ModuleKey, bool>,
}

/// Everything a resolution needs, loaded once per call. Organization trees
/// have dozens of nodes, so folding in memory beats a recursive query.
struct Snapshot {
    hierarchy: OrgHierarchy,
    rows: HashMap<Id<Organization>, BTreeMap<ModuleKey, ModuleState>>,
}

impl Snapshot {
    fn state(&self, key: &ModuleKey, default_enabled: bool, org: Id<Organization>) -> ModuleState {
        let (holder, kind) = match self.hierarchy.tenant_of(org) {
            Some(tenant) => (tenant, SwitchHolder::Tenant),
            None => (org, SwitchHolder::Root),
        };
        let row = self.rows.get(&holder).and_then(|m| m.get(key)).copied();
        module_state(kind, default_enabled, row)
    }

    fn states(
        &self,
        defaults: &BTreeMap<ModuleKey, bool>,
        org: Id<Organization>,
    ) -> BTreeMap<ModuleKey, ModuleState> {
        defaults
            .iter()
            .map(|(key, default)| (key.clone(), self.state(key, *default, org)))
            .collect()
    }
}

impl PgModuleActivationRepository {
    pub fn new(
        pool: PgPool,
        org_reader: Arc<dyn OrganizationReader>,
        defaults: BTreeMap<ModuleKey, bool>,
    ) -> Self {
        Self {
            pool,
            org_reader,
            defaults,
        }
    }

    async fn snapshot(&self) -> Result<Snapshot, RepositoryError> {
        let hierarchy = self.org_reader.hierarchy().await?;
        let rows = sqlx::query!(
            r#"SELECT organization_id, module_key, released, enabled FROM organization_modules"#
        )
        .fetch_all(&self.pool)
        .await?;
        let mut by_org: HashMap<Id<Organization>, BTreeMap<ModuleKey, ModuleState>> =
            HashMap::new();
        for row in rows {
            by_org
                .entry(Id::new(row.organization_id))
                .or_default()
                .insert(
                    ModuleKey::new(&row.module_key)?,
                    ModuleState {
                        released: row.released,
                        enabled: row.enabled,
                    },
                );
        }

        Ok(Snapshot {
            hierarchy,
            rows: by_org,
        })
    }

    async fn orgs_where(
        &self,
        key: &ModuleKey,
        pred: fn(ModuleState) -> bool,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        let Some(default) = self.defaults.get(key).copied() else {
            return Ok(BTreeSet::new());
        };
        let snapshot = self.snapshot().await?;
        let orgs = self.org_reader.all().await?;
        Ok(orgs
            .into_iter()
            .map(|org| org.id)
            .filter(|id| pred(snapshot.state(key, default, *id)))
            .collect())
    }
}

#[async_trait]
impl ModuleActivationReader for PgModuleActivationRepository {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn resolve(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ModuleState>, RepositoryError> {
        let snapshot = self.snapshot().await?;
        // Without this an unknown id resolves to a one-element chain and reads
        // like a real answer made of defaults.
        if !snapshot.hierarchy.knows(org) {
            return Err(RepositoryError::NotFound);
        }
        Ok(snapshot.states(&self.defaults, org))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn active_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        self.orgs_where(key, ModuleState::active).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn released_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        self.orgs_where(key, |state| state.released).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn tenants(
        &self,
    ) -> Result<BTreeMap<Id<Organization>, BTreeMap<ModuleKey, ModuleState>>, RepositoryError> {
        let snapshot = self.snapshot().await?;
        let orgs = self.org_reader.all().await?;
        Ok(orgs
            .into_iter()
            .map(|org| org.id)
            .filter(|id| snapshot.hierarchy.tenant_of(*id) == Some(*id))
            .map(|id| (id, snapshot.states(&self.defaults, id)))
            .collect())
    }
}

#[async_trait]
impl ModuleActivationWriter for PgModuleActivationRepository {
    #[tracing::instrument(level = "info", skip(self, changes))]
    async fn switch(
        &self,
        holder: Id<Organization>,
        changes: &BTreeMap<ModuleKey, bool>,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        for (key, enabled) in changes {
            sqlx::query!(
                r#"INSERT INTO organization_modules (organization_id, module_key, released, enabled)
                   VALUES ($1, $2, TRUE, $3)
                   ON CONFLICT (organization_id, module_key)
                   DO UPDATE SET enabled = EXCLUDED.enabled"#,
                holder.value(),
                key.as_str(),
                enabled
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // The SQL mirrors `released_state`: the first release switches on, a later
    // one keeps the stored switch.
    #[tracing::instrument(level = "info", skip(self, changes))]
    async fn release(
        &self,
        tenant: Id<Organization>,
        changes: &BTreeMap<ModuleKey, bool>,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        for (key, release) in changes {
            if *release {
                sqlx::query!(
                    r#"INSERT INTO organization_modules (organization_id, module_key, released, enabled)
                       VALUES ($1, $2, TRUE, TRUE)
                       ON CONFLICT (organization_id, module_key)
                       DO UPDATE SET released = TRUE"#,
                    tenant.value(),
                    key.as_str()
                )
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query!(
                    r#"UPDATE organization_modules SET released = FALSE
                       WHERE organization_id = $1 AND module_key = $2"#,
                    tenant.value(),
                    key.as_str()
                )
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
