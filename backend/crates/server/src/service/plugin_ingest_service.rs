//! The ingest surface a plugin's own adapter authenticates against with its
//! API key (not a user session) to push tree records into the domain.
//!
//! `upsert_trees` processes each batch entry independently and outside any
//! enclosing transaction: a validation failure, or a conflict specific to one
//! entry, becomes [`IngestStatus::Failed`] for that entry alone and never
//! fails the rest of the batch.

use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    authorization::{AccessContext, Action, EffectivePermissions, Permission, Resource},
    events::DomainEvent,
    plugin::{Plugin, PluginReader, PluginWriter, TreeRefPage},
    shared::{
        coordinates::Coordinate,
        error::ValidationError,
        provenance::{Provenance, ProviderId},
    },
    tree::{PlantingYear, Species, Tree, TreeDraft, TreeNumber, TreeReader, TreeWriter},
};

use super::{
    AuthError, ServiceError, authorization::AuthorizationService, event_bus::EventBus,
    tree_service::TreeDeletion,
};

/// Batch size cap for [`PluginIngestService::upsert_trees`]. Enforced by the
/// HTTP layer (413, code `plugin.batch_too_large`) before a request even
/// reaches the service.
pub const MAX_INGEST_ITEMS: usize = 500;

/// One ingest batch entry, parsed off the wire but not yet turned into domain
/// value objects — that conversion happens per item inside the service so a
/// single malformed entry cannot fail the whole batch.
#[derive(Debug, Clone)]
pub struct TreeIngestItem {
    pub external_id: String,
    pub number: String,
    pub species: String,
    pub planting_year: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub description: Option<String>,
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestStatus {
    Created,
    Updated,
    Unchanged,
    Failed,
}

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub external_id: String,
    pub status: IngestStatus,
    /// Set on every non-`Failed` result.
    pub tree_id: Option<Id<Tree>>,
    /// Set only on `Failed`.
    pub error: Option<String>,
}

pub struct PluginIngestService {
    tree_reader: Arc<dyn TreeReader>,
    tree_writer: Arc<dyn TreeWriter>,
    plugin_reader: Arc<dyn PluginReader>,
    plugin_writer: Arc<dyn PluginWriter>,
    /// Owns the delete flow so it runs through the exact same path as the
    /// regular tree API (`TreeDeleted` published, cluster centroid and
    /// status recalculated) instead of a second, drifting copy of it.
    tree_deletion: Arc<dyn TreeDeletion>,
    event_bus: Arc<dyn EventBus>,
    authorization: Arc<AuthorizationService>,
}

impl PluginIngestService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tree_reader: Arc<dyn TreeReader>,
        tree_writer: Arc<dyn TreeWriter>,
        plugin_reader: Arc<dyn PluginReader>,
        plugin_writer: Arc<dyn PluginWriter>,
        tree_deletion: Arc<dyn TreeDeletion>,
        event_bus: Arc<dyn EventBus>,
        authorization: Arc<AuthorizationService>,
    ) -> Self {
        Self {
            tree_reader,
            tree_writer,
            plugin_reader,
            plugin_writer,
            tree_deletion,
            event_bus,
            authorization,
        }
    }

    /// Builds the plugin's own access context: exactly the grants it was
    /// installed with, scoped to its organization subtree. A plugin key
    /// carries no user identity to resolve roles from, so this does not go
    /// through `AuthorizationService::context_for`.
    async fn context_for(&self, plugin: &Plugin) -> Result<AccessContext, ServiceError> {
        Ok(AccessContext {
            permissions: EffectivePermissions::from_grants(vec![(
                plugin.organization_id(),
                plugin.permissions().clone(),
            )]),
            hierarchy: self.authorization.hierarchy().await?,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn upsert_trees(
        &self,
        plugin: &Plugin,
        items: Vec<TreeIngestItem>,
    ) -> Result<Vec<IngestResult>, ServiceError> {
        let ctx = self.context_for(plugin).await?;
        let org = plugin.organization_id();
        let can_create = ctx.allows_in(Permission::new(Resource::Tree, Action::Create), org);
        let can_update = ctx.allows_in(Permission::new(Resource::Tree, Action::Update), org);
        if !can_create || !can_update {
            return Err(AuthError::Forbidden.into());
        }

        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(self.upsert_one(&ctx, plugin, item).await);
        }
        Ok(results)
    }

    async fn upsert_one(
        &self,
        ctx: &AccessContext,
        plugin: &Plugin,
        item: TreeIngestItem,
    ) -> IngestResult {
        let external_id = item.external_id.clone();
        match self.try_upsert_one(ctx, plugin, item).await {
            Ok((status, tree_id)) => IngestResult {
                external_id,
                status,
                tree_id: Some(tree_id),
                error: None,
            },
            Err(e) => IngestResult {
                external_id,
                status: IngestStatus::Failed,
                tree_id: None,
                error: Some(ingest_failure_message(&e)),
            },
        }
    }

    async fn try_upsert_one(
        &self,
        ctx: &AccessContext,
        plugin: &Plugin,
        item: TreeIngestItem,
    ) -> Result<(IngestStatus, Id<Tree>), ServiceError> {
        // `external_id` is an opaque caller-supplied key that ends up in a
        // btree primary key column (`plugin_tree_refs`); an unbounded value
        // can overflow Postgres' index row-size limit (SQLSTATE 54000, which
        // is unmapped and would otherwise surface as `RepositoryError::Internal`)
        // before any other field is even validated.
        validate_external_id(&item.external_id)?;
        let species = Species::new(item.species)?;
        let tree_number = TreeNumber::new(item.number)?;
        let planting_year = PlantingYear::new(item.planting_year as u32)?;
        let coordinate = Coordinate::new(item.latitude, item.longitude)?;
        // The provenance's provider is always the calling plugin's own slug:
        // a plugin may only stamp its own identity, never assert another's.
        let provider = ProviderId::new(plugin.slug().as_str())?;
        let provenance = Provenance::new(Some(provider), item.additional_info);

        match self
            .plugin_reader
            .tree_ref(plugin.id, &item.external_id)
            .await?
        {
            None => {
                let draft = TreeDraft {
                    planting_year,
                    species,
                    tree_number,
                    coordinate,
                    description: item.description,
                    cluster_id: None,
                    sensor_id: None,
                    provenance,
                    organization_id: plugin.organization_id(),
                };
                let tree = self.tree_writer.save_new(draft).await?;
                if let Err(link_err) = self
                    .plugin_writer
                    .link_tree(plugin.id, &item.external_id, tree.id)
                    .await
                {
                    // The tree is already committed at this point. Whatever
                    // the reason `link_tree` failed, leaving it behind would
                    // orphan a row a retried import recreates forever, so it
                    // is deleted before this entry reports failed; no
                    // `TreeCreated` is published for a tree that no longer
                    // exists.
                    if let Err(delete_err) = self.tree_writer.delete(tree.id).await {
                        tracing::error!(
                            tree.id = %tree.id,
                            error = %delete_err,
                            "failed to delete orphaned tree after a failed link_tree"
                        );
                    }
                    return Err(match link_err {
                        // `link_tree` upserts on the primary key (plugin_id,
                        // external_id), so a repeated external_id never
                        // reaches here. Only the table's second unique index
                        // (plugin_id, tree_id) can still reject this insert,
                        // meaning the tree is already tracked under a
                        // different external_id for this same plugin — the
                        // caller's data problem, not an infrastructure
                        // failure.
                        RepositoryError::AlreadyExists(_) => {
                            link_conflict_error(&item.external_id, tree.id)
                        }
                        other => other.into(),
                    });
                }
                self.event_bus
                    .publish(DomainEvent::TreeCreated {
                        tree_id: tree.id,
                        cluster_id: tree.cluster_id(),
                        sensor_id: tree.sensor_id().cloned(),
                    })
                    .await;
                Ok((IngestStatus::Created, tree.id))
            }
            Some(r) => {
                let mut tree = self.tree_reader.by_id(r.tree_id).await?;
                // The plugin's grant is checked against the tree's *current*
                // organization, not the plugin's own: a tree created by this
                // plugin can be moved elsewhere by an admin afterwards (e.g.
                // via `POST /trees/{id}/organization`), and the reference in
                // `plugin_tree_refs` survives that move untouched.
                if !ctx.allows_in(
                    Permission::new(Resource::Tree, Action::Update),
                    tree.organization_id(),
                ) {
                    return Err(AuthError::Forbidden.into());
                }
                if tree.species == species
                    && tree.tree_number == tree_number
                    && tree.planting_year == planting_year
                    && tree.coordinate == coordinate
                    && tree.description == item.description
                    && tree.provenance() == &provenance
                {
                    return Ok((IngestStatus::Unchanged, tree.id));
                }
                let events = tree.replace_details(
                    species,
                    tree_number,
                    planting_year,
                    coordinate,
                    item.description,
                    provenance,
                );
                self.tree_writer.save(&tree).await?;
                self.event_bus.publish_all(events).await;
                Ok((IngestStatus::Updated, tree.id))
            }
        }
    }

    /// Deletes the tree the plugin's `external_id` resolves to, through
    /// `TreeService::delete` so `TreeDeleted` fires and cluster centroid and
    /// status get recalculated exactly as for a user-initiated deletion.
    ///
    /// No separate unlink call afterwards: `plugin_tree_refs.tree_id` cascades
    /// on delete, so the reference row is already gone once the tree is.
    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn delete_tree(
        &self,
        plugin: &Plugin,
        external_id: &str,
    ) -> Result<(), ServiceError> {
        let ctx = self.context_for(plugin).await?;
        let tree_ref = self
            .plugin_reader
            .tree_ref(plugin.id, external_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;
        // Same reasoning as the update path: check against the tree's actual
        // organization, which may no longer be the plugin's own since a tree
        // can be transferred after the plugin created it.
        let tree = self.tree_reader.by_id(tree_ref.tree_id).await?;
        if !ctx.allows_in(
            Permission::new(Resource::Tree, Action::Delete),
            tree.organization_id(),
        ) {
            return Err(AuthError::Forbidden.into());
        }
        self.tree_deletion.delete(tree_ref.tree_id).await
    }

    /// Keyset page over `plugin`'s own external_id-to-tree mappings.
    #[tracing::instrument(level = "debug", skip_all, fields(plugin.id = %plugin.id))]
    pub async fn list_refs(
        &self,
        plugin: &Plugin,
        after: Option<&str>,
        limit: u32,
    ) -> Result<TreeRefPage, ServiceError> {
        Ok(self
            .plugin_reader
            .tree_refs(plugin.id, after, limit)
            .await?)
    }
}

/// Turns the (plugin_id, tree_id) unique-index rejection from `link_tree`
/// into a message naming the offending tree, kept separate from
/// `try_upsert_one` so it is unit-testable without a database.
fn link_conflict_error(external_id: &str, tree_id: Id<Tree>) -> ServiceError {
    ServiceError::InvalidInput(format!(
        "external_id '{external_id}' cannot be linked: this plugin already tracks tree {tree_id} under a different external_id"
    ))
}

/// `external_id` has no domain-level newtype (it is an opaque key the plugin
/// invents), but it still lands in a btree primary key column, so an
/// unbounded value is a real, remotely-triggerable failure mode rather than
/// a defensive nicety.
fn validate_external_id(external_id: &str) -> Result<(), ValidationError> {
    let len = external_id.chars().count();
    if len == 0 {
        return Err(ValidationError::EmptyString {
            field: "plugin.external_id",
        });
    }
    if len > 255 {
        return Err(ValidationError::TooLong {
            field: "plugin.external_id",
            max: 255,
            got: len,
        });
    }
    Ok(())
}

/// Per-entry failure text for the ingest response body. A `ValidationError`
/// keeps its own message — that is the actionable part for an adapter
/// author — but everything backed by a repository failure collapses to the
/// same generic wording `http/v1/error.rs` uses for a 5xx body, with the real
/// cause logged instead: this response is still a 200, and it must not leak
/// driver or constraint detail the way a failing request already refuses to.
fn ingest_failure_message(e: &ServiceError) -> String {
    match e {
        ServiceError::Validation(inner) => inner.to_string(),
        ServiceError::Auth(inner) => inner.to_string(),
        ServiceError::InvalidInput(msg) => msg.clone(),
        ServiceError::Repository(inner) => {
            match inner {
                RepositoryError::DataIntegrity(_) | RepositoryError::Internal(_) => {
                    tracing::error!(error = %inner, kind = "repository", "plugin ingest entry failed")
                }
                _ => {
                    tracing::warn!(error = %inner, kind = "repository", "plugin ingest entry failed")
                }
            }
            inner.generic_message().to_string()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};
    use std::sync::Mutex;

    use domain::authorization::{OrgHierarchy, Permission};
    use domain::organization::{Organization, OrganizationReader};
    use domain::plugin::{PluginDraft, PluginKeyHash, PluginSlug, PluginSnapshot, TreeRef};
    use domain::role::{Role, RoleReader};
    use domain::sensor::SensorId;
    use domain::shared::watering_status::WateringStatus;
    use domain::tree::TreeSnapshot;
    use serde_json::json;
    use uuid::Uuid;

    use crate::service::event_bus::NoopEventBus;

    #[derive(Default)]
    struct FakeTreeRepo {
        rows: Mutex<HashMap<Uuid, Tree>>,
    }

    #[async_trait::async_trait]
    impl TreeReader for FakeTreeRepo {
        async fn by_id(&self, id: Id<Tree>) -> Result<Tree, RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .get(&id.value())
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }
        async fn by_ids(&self, _ids: &[Id<Tree>]) -> Result<Vec<Tree>, RepositoryError> {
            unimplemented!()
        }
        async fn by_sensor_id(
            &self,
            _sensor_id: &SensorId,
        ) -> Result<Option<Tree>, RepositoryError> {
            unimplemented!()
        }
        async fn by_cluster_id(
            &self,
            _cluster_id: Id<domain::cluster::TreeCluster>,
        ) -> Result<Vec<Tree>, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_id(
            &self,
            _id: Id<Tree>,
        ) -> Result<domain::tree::TreeView, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_ids(
            &self,
            _ids: &[Id<Tree>],
        ) -> Result<Vec<domain::tree::TreeView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_sensor_id(
            &self,
            _sensor_id: &SensorId,
        ) -> Result<Option<domain::tree::TreeView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_search(
            &self,
            _query: domain::tree::TreeSearchQuery,
            _pagination: domain::shared::pagination::Pagination,
        ) -> Result<domain::shared::pagination::Page<domain::tree::TreeView>, RepositoryError>
        {
            unimplemented!()
        }
        async fn view_nearest(
            &self,
            _coord: Coordinate,
            _radius: domain::shared::distance::Distance,
            _limit: u32,
            _visible: domain::authorization::Visibility,
        ) -> Result<Vec<domain::tree::TreeViewWithDistance>, RepositoryError> {
            unimplemented!()
        }
        async fn find_nearest(
            &self,
            _coord: Coordinate,
            _radius: domain::shared::distance::Distance,
        ) -> Result<Option<Tree>, RepositoryError> {
            unimplemented!()
        }
        async fn view_markers(
            &self,
            _query: domain::tree::TreeSearchQuery,
        ) -> Result<Vec<domain::tree::TreeMarker>, RepositoryError> {
            unimplemented!()
        }
        async fn distinct_planting_years(
            &self,
            _visible: domain::authorization::Visibility,
        ) -> Result<Vec<PlantingYear>, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl TreeWriter for FakeTreeRepo {
        async fn save_new(&self, draft: TreeDraft) -> Result<Tree, RepositoryError> {
            let id: Id<Tree> = Id::new_v7();
            let tree = Tree::reconstitute(TreeSnapshot {
                id: id.value(),
                planting_year: draft.planting_year.year() as i32,
                species: draft.species.as_str().to_string(),
                tree_number: draft.tree_number.as_str().to_string(),
                latitude: draft.coordinate.latitude(),
                longitude: draft.coordinate.longitude(),
                description: draft.description,
                last_watered: None,
                cluster_id: draft.cluster_id.map(|c| c.value()),
                sensor_id: draft.sensor_id.map(|s| s.as_str().to_string()),
                watering_status: WateringStatus::Unknown,
                provider: draft.provenance.provider().map(|p| p.as_str().to_string()),
                additional_info: draft.provenance.additional_info().cloned(),
                organization_id: draft.organization_id.value(),
            });
            self.rows.lock().unwrap().insert(id.value(), tree.clone());
            Ok(tree)
        }

        async fn save(&self, tree: &Tree) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .insert(tree.id.value(), tree.clone());
            Ok(())
        }

        async fn delete(&self, id: Id<Tree>) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .remove(&id.value())
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)
        }
    }

    /// In production the port is implemented by `TreeService`, so the ingest
    /// delete really runs the regular tree flow; `plugin_ingest.rs`
    /// (`delete_removes_tree_and_ref`) covers that end to end. Here the repo
    /// stands in for it, so these tests can pin the ingest service's own
    /// behaviour — ref lookup, per-tree authorization, delegation — without
    /// building a `TreeService` out of fakes it never calls.
    #[async_trait::async_trait]
    impl TreeDeletion for FakeTreeRepo {
        async fn delete(&self, id: Id<Tree>) -> Result<(), ServiceError> {
            TreeWriter::delete(self, id).await?;
            Ok(())
        }
    }

    impl FakeTreeRepo {
        fn contains(&self, id: Id<Tree>) -> bool {
            self.rows.lock().unwrap().contains_key(&id.value())
        }

        /// Test-only: simulates an admin moving the tree to another
        /// organization (e.g. via `PATCH /trees/{id}/organization`),
        /// independently of and after the plugin's own import.
        fn transfer(&self, id: Id<Tree>, target: Id<Organization>) {
            let mut rows = self.rows.lock().unwrap();
            let tree = rows.get_mut(&id.value()).expect("tree exists");
            tree.transfer_to(target);
        }
    }

    /// Which error `link_tree` manufactures for its denied external_id.
    enum LinkDenial {
        /// Models the second unique index (plugin_id, tree_id) rejecting a
        /// link: the tree is already tracked under a different external_id.
        AlreadyExists,
        /// Models an unrelated repository failure (e.g. the real
        /// `index row size ... exceeds btree ... maximum` a too-long
        /// external_id triggers, which is unmapped and surfaces as
        /// `RepositoryError::Internal`).
        Internal,
    }

    /// Only implements the methods `PluginIngestService` actually calls;
    /// anything else panics loudly so a new call site cannot pass unnoticed.
    struct FakePluginRepo {
        refs: Mutex<HashMap<(Uuid, String), TreeRef>>,
        /// `link_tree` returns the matching error for this exact external_id
        /// once, then falls back to succeeding.
        deny_link: Option<(String, LinkDenial)>,
    }

    impl FakePluginRepo {
        fn new() -> Self {
            Self {
                refs: Mutex::new(HashMap::new()),
                deny_link: None,
            }
        }

        fn denying_link_for(external_id: &str) -> Self {
            Self {
                refs: Mutex::new(HashMap::new()),
                deny_link: Some((external_id.to_string(), LinkDenial::AlreadyExists)),
            }
        }

        fn denying_link_internally_for(external_id: &str) -> Self {
            Self {
                refs: Mutex::new(HashMap::new()),
                deny_link: Some((external_id.to_string(), LinkDenial::Internal)),
            }
        }
    }

    #[async_trait::async_trait]
    impl PluginReader for FakePluginRepo {
        async fn by_id(&self, _id: Id<Plugin>) -> Result<Plugin, RepositoryError> {
            unimplemented!()
        }
        async fn by_slug(&self, _slug: &PluginSlug) -> Result<Plugin, RepositoryError> {
            unimplemented!()
        }
        async fn all(&self) -> Result<Vec<domain::plugin::PluginView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_slug(
            &self,
            _slug: &PluginSlug,
        ) -> Result<domain::plugin::PluginView, RepositoryError> {
            unimplemented!()
        }
        async fn last_seen_at(
            &self,
            _id: Id<Plugin>,
        ) -> Result<Option<chrono::DateTime<chrono::Utc>>, RepositoryError> {
            unimplemented!()
        }
        async fn tree_ref(
            &self,
            plugin: Id<Plugin>,
            external_id: &str,
        ) -> Result<Option<TreeRef>, RepositoryError> {
            Ok(self
                .refs
                .lock()
                .unwrap()
                .get(&(plugin.value(), external_id.to_string()))
                .cloned())
        }
        async fn tree_refs(
            &self,
            plugin: Id<Plugin>,
            after: Option<&str>,
            limit: u32,
        ) -> Result<TreeRefPage, RepositoryError> {
            let mut items: Vec<TreeRef> = self
                .refs
                .lock()
                .unwrap()
                .iter()
                .filter(|((p, _), _)| *p == plugin.value())
                .map(|(_, r)| r.clone())
                .collect();
            items.sort_by(|a, b| a.external_id.cmp(&b.external_id));
            if let Some(after) = after {
                items.retain(|r| r.external_id.as_str() > after);
            }
            items.truncate(limit as usize);
            Ok(TreeRefPage {
                items,
                next_cursor: None,
            })
        }
    }

    #[async_trait::async_trait]
    impl PluginWriter for FakePluginRepo {
        async fn save_new(
            &self,
            _id: Id<Plugin>,
            _draft: PluginDraft,
            _key_hash: Option<PluginKeyHash>,
        ) -> Result<Plugin, RepositoryError> {
            unimplemented!()
        }
        async fn save(&self, _plugin: &Plugin) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn delete(&self, _id: Id<Plugin>) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn touch_last_seen(&self, _id: Id<Plugin>) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn link_tree(
            &self,
            plugin: Id<Plugin>,
            external_id: &str,
            tree: Id<Tree>,
        ) -> Result<(), RepositoryError> {
            if let Some((denied, kind)) = &self.deny_link
                && denied == external_id
            {
                return Err(match kind {
                    LinkDenial::AlreadyExists => RepositoryError::AlreadyExists(
                        "plugin_tree_refs_plugin_id_tree_id_key".into(),
                    ),
                    LinkDenial::Internal => RepositoryError::Internal(
                        "index row size 6432 exceeds btree version 4 maximum 2704".into(),
                    ),
                });
            }
            self.refs.lock().unwrap().insert(
                (plugin.value(), external_id.to_string()),
                TreeRef {
                    external_id: external_id.to_string(),
                    tree_id: tree,
                },
            );
            Ok(())
        }
        async fn unlink_tree(
            &self,
            plugin: Id<Plugin>,
            external_id: &str,
        ) -> Result<(), RepositoryError> {
            self.refs
                .lock()
                .unwrap()
                .remove(&(plugin.value(), external_id.to_string()))
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)
        }
    }

    struct UnrestrictedOrgs;

    #[async_trait::async_trait]
    impl OrganizationReader for UnrestrictedOrgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, _id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs(Vec::new()))
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(HashMap::new())
        }
    }

    struct NoRoles;

    #[async_trait::async_trait]
    impl RoleReader for NoRoles {
        async fn by_id(&self, _id: Id<Role>) -> Result<Role, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn by_organization(
            &self,
            _org: Id<Organization>,
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_organizations(
            &self,
            _orgs: &[Id<Organization>],
        ) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn templates(&self) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn roles_for_user(&self, _user_id: Uuid) -> Result<Vec<Role>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn roles_for_users(
            &self,
            _ids: &[Uuid],
        ) -> Result<Vec<(Uuid, Role)>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn user_ids_with_role(
            &self,
            _role_id: Id<Role>,
        ) -> Result<Vec<Uuid>, RepositoryError> {
            Ok(Vec::new())
        }
    }

    fn plugin_fixture(permissions: BTreeSet<Permission>) -> Plugin {
        Plugin::reconstitute(PluginSnapshot {
            id: Uuid::now_v7(),
            slug: "kataster".into(),
            name: "Kataster".into(),
            description: None,
            frontend_mode: "none".into(),
            frontend_target: None,
            organization_id: Uuid::now_v7(),
            permissions: permissions.iter().map(|p| p.to_string()).collect(),
            required_permissions: Vec::new(),
            enabled: true,
            key_hash: None,
            last_seen_at: None,
        })
    }

    fn full_permissions() -> BTreeSet<Permission> {
        BTreeSet::from([
            Permission::new(Resource::Tree, Action::Create),
            Permission::new(Resource::Tree, Action::Update),
            Permission::new(Resource::Tree, Action::Delete),
        ])
    }

    /// Also returns the tree repo so tests can inspect what actually got
    /// written (e.g. that a failed link left no orphaned tree behind).
    fn service_with(plugin_repo: FakePluginRepo) -> (PluginIngestService, Arc<FakeTreeRepo>) {
        let tree_repo = Arc::new(FakeTreeRepo::default());
        let plugin_repo = Arc::new(plugin_repo);
        let event_bus = Arc::new(NoopEventBus);
        let authorization = Arc::new(AuthorizationService::new(
            Arc::new(UnrestrictedOrgs),
            Arc::new(NoRoles),
            true,
        ));
        let svc = PluginIngestService::new(
            tree_repo.clone(),
            tree_repo.clone(),
            plugin_repo.clone(),
            plugin_repo,
            tree_repo.clone(),
            event_bus,
            authorization,
        );
        (svc, tree_repo)
    }

    fn item(external_id: &str, species: &str) -> TreeIngestItem {
        TreeIngestItem {
            external_id: external_id.into(),
            number: "FL-001".into(),
            species: species.into(),
            planting_year: 1998,
            latitude: 54.7836,
            longitude: 9.4321,
            description: None,
            additional_info: Some(json!({ "objectid": 12345 })),
        }
    }

    #[tokio::test]
    async fn first_run_creates() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        let results = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, IngestStatus::Created);
        assert!(results[0].tree_id.is_some());
    }

    #[tokio::test]
    async fn missing_permission_is_forbidden() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(BTreeSet::from([Permission::new(
            Resource::Tree,
            Action::Read,
        )]));

        let err = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Auth(AuthError::Forbidden)));
    }

    #[tokio::test]
    async fn a_bad_entry_fails_alone() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        let results = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur"), item("2", "")])
            .await
            .unwrap();

        assert_eq!(results[0].status, IngestStatus::Created);
        assert_eq!(results[1].status, IngestStatus::Failed);
        assert!(results[1].error.as_ref().unwrap().contains("species"));
    }

    /// `external_id` is checked before the first database call (`tree_ref`),
    /// so an oversized key never reaches the btree primary key it would
    /// otherwise overflow.
    #[tokio::test]
    async fn an_oversized_external_id_fails_before_touching_the_database() {
        let (svc, tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());
        let mut too_long = item("x", "Quercus robur");
        too_long.external_id = "x".repeat(256);

        let results = svc.upsert_trees(&plugin, vec![too_long]).await.unwrap();

        assert_eq!(results[0].status, IngestStatus::Failed);
        assert!(
            results[0]
                .error
                .as_ref()
                .unwrap()
                .contains("plugin.external_id"),
            "got: {:?}",
            results[0].error
        );
        assert_eq!(tree_repo.rows.lock().unwrap().len(), 0);
    }

    /// The scenario the `link_tree` primary key alone cannot rule out: a
    /// second unique index on (plugin_id, tree_id) rejects linking a tree
    /// already tracked under a different external_id for the same plugin.
    /// The batch keeps going; only this entry reports `Failed`.
    #[tokio::test]
    async fn a_tree_id_conflict_on_link_fails_only_that_entry() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::denying_link_for("2"));
        let plugin = plugin_fixture(full_permissions());

        let results = svc
            .upsert_trees(
                &plugin,
                vec![item("1", "Quercus robur"), item("2", "Tilia cordata")],
            )
            .await
            .unwrap();

        assert_eq!(results[0].status, IngestStatus::Created);
        assert_eq!(results[1].status, IngestStatus::Failed);
        assert!(
            results[1]
                .error
                .as_ref()
                .unwrap()
                .contains("already tracks tree"),
            "got: {:?}",
            results[1].error
        );
    }

    #[tokio::test]
    async fn second_run_updates_and_third_is_unchanged() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        svc.upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap();
        let updated = svc
            .upsert_trees(&plugin, vec![item("1", "Tilia cordata")])
            .await
            .unwrap();
        assert_eq!(updated[0].status, IngestStatus::Updated);

        let again = svc
            .upsert_trees(&plugin, vec![item("1", "Tilia cordata")])
            .await
            .unwrap();
        assert_eq!(again[0].status, IngestStatus::Unchanged);
    }

    #[tokio::test]
    async fn delete_missing_ref_is_not_found() {
        let (svc, _tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        let err = svc.delete_tree(&plugin, "unknown").await.unwrap_err();
        assert!(matches!(
            err,
            ServiceError::Repository(RepositoryError::NotFound)
        ));
    }

    #[tokio::test]
    async fn delete_removes_the_tree() {
        let (svc, tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        let created = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap();
        let tree_id = created[0].tree_id.unwrap();

        svc.delete_tree(&plugin, "1").await.unwrap();

        assert!(!tree_repo.contains(tree_id));
    }

    /// Companion to `a_tree_id_conflict_on_link_fails_only_that_entry`: any
    /// other `link_tree` failure must not leave the tree it was meant to
    /// link behind, and the reported error must not carry the driver detail.
    #[tokio::test]
    async fn a_non_conflict_link_failure_deletes_the_orphan_and_hides_the_detail() {
        let (svc, tree_repo) = service_with(FakePluginRepo::denying_link_internally_for("1"));
        let plugin = plugin_fixture(full_permissions());

        let results = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap();

        assert_eq!(results[0].status, IngestStatus::Failed);
        assert!(results[0].tree_id.is_none());
        let message = results[0].error.as_ref().unwrap();
        assert_eq!(message, "internal server error");
        assert!(
            !message.contains("btree") && !message.contains("index row size"),
            "driver detail must not reach the client, got: {message}"
        );
        assert_eq!(
            tree_repo.rows.lock().unwrap().len(),
            0,
            "the tree created before the failed link must not survive"
        );
    }

    /// The critical regression: a tree the plugin created can be moved to
    /// another organization afterwards (e.g. by an admin, since a clusterless
    /// tree has no other lock on it). The plugin's own reference in
    /// `plugin_tree_refs` survives that move untouched, so both the update
    /// and the delete path must check the tree's *current* organization
    /// rather than trusting the plugin's own.
    #[tokio::test]
    async fn transferring_the_tree_forbids_further_update_and_delete() {
        let (svc, tree_repo) = service_with(FakePluginRepo::new());
        let plugin = plugin_fixture(full_permissions());

        let created = svc
            .upsert_trees(&plugin, vec![item("1", "Quercus robur")])
            .await
            .unwrap();
        let tree_id = created[0].tree_id.unwrap();

        tree_repo.transfer(tree_id, Id::new_v7());

        let updated = svc
            .upsert_trees(&plugin, vec![item("1", "Tilia cordata")])
            .await
            .unwrap();
        assert_eq!(updated[0].status, IngestStatus::Failed);
        assert!(
            updated[0].error.as_ref().unwrap().contains("forbidden"),
            "got: {:?}",
            updated[0].error
        );

        let delete_err = svc.delete_tree(&plugin, "1").await.unwrap_err();
        assert!(matches!(
            delete_err,
            ServiceError::Auth(AuthError::Forbidden)
        ));
    }
}
