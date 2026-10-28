use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use domain::{
    Id, RepositoryError,
    authorization::OrgHierarchy,
    module::{
        ActivationLevel, ActivationResolution, ModuleActivationReader, ModuleActivationWriter,
        ModuleKey, OwnSwitch, resolve_activation,
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
    locked: HashSet<Id<Organization>>,
    own: HashMap<Id<Organization>, BTreeMap<ModuleKey, bool>>,
}

impl Snapshot {
    fn resolve(
        &self,
        key: &ModuleKey,
        default_enabled: bool,
        org: Id<Organization>,
    ) -> ActivationResolution {
        let chain: Vec<ActivationLevel> = self
            .hierarchy
            .ancestors_from_root(org)
            .into_iter()
            .map(|id| ActivationLevel {
                organization_id: id,
                enabled: self.own.get(&id).and_then(|m| m.get(key)).copied(),
                descendants_may_override: !self.locked.contains(&id),
            })
            .collect();
        resolve_activation(default_enabled, &chain, org)
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
        let locked = sqlx::query_scalar!(
            r#"SELECT organization_id FROM organization_settings
               WHERE descendants_may_override = false"#
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(Id::new)
        .collect();

        let rows = sqlx::query!(
            r#"SELECT organization_id, module_key, enabled FROM organization_modules"#
        )
        .fetch_all(&self.pool)
        .await?;
        let mut own: HashMap<Id<Organization>, BTreeMap<ModuleKey, bool>> = HashMap::new();
        for row in rows {
            own.entry(Id::new(row.organization_id))
                .or_default()
                .insert(ModuleKey::new(&row.module_key)?, row.enabled);
        }

        Ok(Snapshot {
            hierarchy,
            locked,
            own,
        })
    }
}

#[async_trait]
impl ModuleActivationReader for PgModuleActivationRepository {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn own(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, OwnSwitch>, RepositoryError> {
        let rows = sqlx::query!(
            r#"SELECT module_key, enabled, set_by FROM organization_modules
               WHERE organization_id = $1"#,
            org.value()
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok((
                    ModuleKey::new(&row.module_key)?,
                    OwnSwitch {
                        enabled: row.enabled,
                        set_by: Id::new(row.set_by),
                    },
                ))
            })
            .collect()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn resolve(
        &self,
        org: Id<Organization>,
    ) -> Result<BTreeMap<ModuleKey, ActivationResolution>, RepositoryError> {
        let snapshot = self.snapshot().await?;
        // Without this an unknown id resolves to a one-element chain and reads
        // like a real answer made of defaults.
        if !snapshot.hierarchy.knows(org) {
            return Err(RepositoryError::NotFound);
        }
        Ok(self
            .defaults
            .iter()
            .map(|(key, default)| (key.clone(), snapshot.resolve(key, *default, org)))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn active_orgs(
        &self,
        key: &ModuleKey,
    ) -> Result<BTreeSet<Id<Organization>>, RepositoryError> {
        let Some(default) = self.defaults.get(key).copied() else {
            return Ok(BTreeSet::new());
        };
        let snapshot = self.snapshot().await?;
        let orgs = self.org_reader.all().await?;
        Ok(orgs
            .into_iter()
            .map(|org| org.id)
            .filter(|id| snapshot.resolve(key, default, *id).enabled)
            .collect())
    }
}

#[async_trait]
impl ModuleActivationWriter for PgModuleActivationRepository {
    #[tracing::instrument(level = "info", skip(self, changes))]
    async fn apply(
        &self,
        org: Id<Organization>,
        changes: &BTreeMap<ModuleKey, Option<bool>>,
        set_by: Id<Organization>,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await?;
        for (key, change) in changes {
            match change {
                Some(enabled) => {
                    sqlx::query!(
                        r#"INSERT INTO organization_modules
                               (organization_id, module_key, enabled, set_by)
                           VALUES ($1, $2, $3, $4)
                           ON CONFLICT (organization_id, module_key)
                           DO UPDATE SET enabled = EXCLUDED.enabled, set_by = EXCLUDED.set_by"#,
                        org.value(),
                        key.as_str(),
                        enabled,
                        set_by.value()
                    )
                    .execute(&mut *tx)
                    .await?;
                }
                None => {
                    sqlx::query!(
                        r#"DELETE FROM organization_modules
                           WHERE organization_id = $1 AND module_key = $2"#,
                        org.value(),
                        key.as_str()
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
        tx.commit().await?;
        Ok(())
    }
}
