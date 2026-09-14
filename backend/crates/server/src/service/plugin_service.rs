//! Application service that installs, updates and retires plugins.
//!
//! Authorization is enforced here rather than left to the HTTP layer: every
//! mutation is scoped to the plugin's own organization, which only this
//! service can resolve after loading the aggregate. Every operation that
//! grants the plugin reach, or hands out a credential that exercises it,
//! additionally requires the actor to hold a superset of the plugin's own
//! permissions. `required_permissions` is exempt, since it restricts who may
//! open the plugin's view rather than granting the plugin anything.

use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use domain::{
    Id,
    authorization::{Action, Permission, Resource},
    plugin::{
        Plugin, PluginDraft, PluginFrontend, PluginKeyHash, PluginName, PluginReader, PluginSlug,
        PluginView, PluginWriter,
    },
};

use super::{AuthError, ServiceError, authorization::AuthorizationService};

/// Mints the plaintext key shown once to the operator plus the hash to
/// persist. A CSPRNG and a digest are adapter concerns, so the service takes
/// them as a port instead of reaching into `infra` and dragging server-side
/// machinery into a layer that must stay portable.
pub trait PluginKeyFactory: Send + Sync {
    fn generate(&self, plugin: Id<Plugin>) -> (String, PluginKeyHash);
}

/// A pending change to an installed plugin, expressed field by field so an
/// omitted field leaves the aggregate untouched.
pub struct PluginChange {
    pub name: Option<PluginName>,
    pub description: Option<Option<String>>,
    pub frontend: Option<PluginFrontend>,
    pub permissions: Option<BTreeSet<Permission>>,
    pub required_permissions: Option<BTreeSet<Permission>>,
    pub enabled: Option<bool>,
}

pub struct PluginService {
    reader: Arc<dyn PluginReader>,
    writer: Arc<dyn PluginWriter>,
    authorization: Arc<AuthorizationService>,
    keys: Arc<dyn PluginKeyFactory>,
}

impl PluginService {
    pub fn new(
        reader: Arc<dyn PluginReader>,
        writer: Arc<dyn PluginWriter>,
        authorization: Arc<AuthorizationService>,
        keys: Arc<dyn PluginKeyFactory>,
    ) -> Self {
        Self {
            reader,
            writer,
            authorization,
            keys,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(&self, actor: Uuid) -> Result<Vec<PluginView>, ServiceError> {
        let visibility = self
            .authorization
            .visible_orgs_for(actor, Permission::new(Resource::Plugin, Action::Read))
            .await?;
        Ok(self
            .reader
            .all()
            .await?
            .into_iter()
            .filter(|view| visibility.allows(view.organization_id))
            .collect())
    }

    /// Unauthenticated by design: resolving a plugin by slug has no acting
    /// user to check (the ingest path authenticates via the plugin's own key,
    /// not a user token).
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn by_slug(&self, slug: &PluginSlug) -> Result<PluginView, ServiceError> {
        Ok(self.reader.view_by_slug(slug).await?)
    }

    /// Resolves a plugin for someone who wants to open its view. The gate is
    /// the plugin's `required_permissions` (see
    /// [`AuthorizationService::require_plugin_view`]), so a user who never
    /// administers plugins still reaches the view they are meant to work with.
    /// A disabled plugin is refused, but only after the access check, so the
    /// refusal cannot be used to probe which slugs exist.
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn view_for(
        &self,
        actor: Uuid,
        slug: &PluginSlug,
    ) -> Result<PluginView, ServiceError> {
        let plugin = self.reader.by_slug(slug).await?;
        self.authorization
            .require_plugin_view(
                actor,
                plugin.required_permissions(),
                plugin.organization_id(),
            )
            .await?;
        if !plugin.enabled() {
            return Err(AuthError::PluginDisabled.into());
        }
        Ok(PluginView::from_aggregate(&plugin, None))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn install(
        &self,
        actor: Uuid,
        draft: PluginDraft,
    ) -> Result<(PluginView, String), ServiceError> {
        self.authorization
            .require(
                actor,
                Permission::new(Resource::Plugin, Action::Create),
                draft.organization_id,
            )
            .await?;
        self.authorization
            .require_superset(actor, &draft.permissions, draft.organization_id)
            .await?;

        let id = Id::<Plugin>::new_v7();
        let (plaintext, hash) = self.keys.generate(id);
        let plugin = self.writer.save_new(id, draft, Some(hash)).await?;
        Ok((PluginView::from_aggregate(&plugin, None), plaintext))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update(
        &self,
        actor: Uuid,
        slug: &PluginSlug,
        change: PluginChange,
    ) -> Result<PluginView, ServiceError> {
        let mut plugin = self.reader.by_slug(slug).await?;
        let org = plugin.organization_id();
        self.authorization
            .require(
                actor,
                Permission::new(Resource::Plugin, Action::Update),
                org,
            )
            .await?;
        // Two changes put the plugin's rights into use: widening the set, and
        // arming a plugin that was installed disabled. Both demand that the
        // actor holds those rights themselves; a rename must not, or an admin
        // without `tree:delete` could no longer correct a typo.
        if change.permissions.is_some() || change.enabled == Some(true) {
            let effective = change
                .permissions
                .as_ref()
                .unwrap_or_else(|| plugin.permissions());
            self.authorization
                .require_superset(actor, effective, org)
                .await?;
        }

        if let Some(name) = change.name {
            plugin.rename(name);
        }
        if let Some(description) = change.description {
            plugin.set_description(description);
        }
        if let Some(frontend) = change.frontend {
            plugin.set_frontend(frontend);
        }
        if let Some(permissions) = change.permissions {
            plugin.replace_permissions(permissions);
        }
        if let Some(required_permissions) = change.required_permissions {
            plugin.replace_required_permissions(required_permissions);
        }
        if let Some(enabled) = change.enabled {
            if enabled {
                plugin.enable();
            } else {
                plugin.disable();
            }
        }

        self.writer.save(&plugin).await?;
        let last_seen_at = self.reader.last_seen_at(plugin.id).await?;
        Ok(PluginView::from_aggregate(&plugin, last_seen_at))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn rotate_key(&self, actor: Uuid, slug: &PluginSlug) -> Result<String, ServiceError> {
        let mut plugin = self.reader.by_slug(slug).await?;
        let org = plugin.organization_id();
        self.authorization
            .require(
                actor,
                Permission::new(Resource::Plugin, Action::Update),
                org,
            )
            .await?;
        // A rotation hands the caller a working credential for this plugin,
        // so it grants everything the plugin may do just as install does.
        // Without this the superset check on install and update is defeated
        // by rotating the key of an already privileged plugin and calling the
        // ingest API with it directly.
        self.authorization
            .require_superset(actor, plugin.permissions(), org)
            .await?;

        let (plaintext, hash) = self.keys.generate(plugin.id);
        plugin.rotate_credential(hash);
        self.writer.save(&plugin).await?;
        Ok(plaintext)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn uninstall(&self, actor: Uuid, slug: &PluginSlug) -> Result<(), ServiceError> {
        let plugin = self.reader.by_slug(slug).await?;
        self.authorization
            .require(
                actor,
                Permission::new(Resource::Plugin, Action::Delete),
                plugin.organization_id(),
            )
            .await?;
        self.writer.delete(plugin.id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::plugin_key::{self, RandomPluginKeyFactory};
    use domain::authorization::{Action, Resource};

    #[tokio::test]
    async fn install_returns_a_plaintext_key_once() {
        let svc = service_with_unrestricted_auth();
        let (view, key) = svc.install(Uuid::nil(), draft("acme")).await.unwrap();

        assert!(key.starts_with("gep_"));
        assert!(view.has_credential);
        let reloaded = svc
            .by_slug(&PluginSlug::new("acme").unwrap())
            .await
            .unwrap();
        assert!(reloaded.has_credential, "the hash is persisted");
    }

    #[tokio::test]
    async fn install_returns_a_key_matching_the_persisted_id() {
        // The key embeds an id minted before the row exists (`install` mints
        // it to generate the key, then hands it to `save_new`); this catches
        // a repository silently minting its own id instead of using the one
        // it was given, which would hand out a key that can never resolve
        // back to a real plugin.
        let svc = service_with_unrestricted_auth();
        let (view, key) = svc.install(Uuid::nil(), draft("acme")).await.unwrap();

        let (parsed_id, _secret) = plugin_key::parse_key(&key).expect("key must parse");
        assert_eq!(
            parsed_id,
            view.id.value(),
            "the key must reference the plugin that was actually persisted"
        );
    }

    #[tokio::test]
    async fn install_rejects_permissions_the_actor_lacks() {
        let svc = service_with_auth_allowing(&[Permission::new(Resource::Plugin, Action::Create)]);
        let mut d = draft("acme");
        d.permissions = BTreeSet::from([Permission::new(Resource::Tree, Action::Delete)]);

        let err = svc.install(Uuid::new_v4(), d).await.unwrap_err();
        assert!(matches!(err, ServiceError::Auth(_)));
    }

    #[tokio::test]
    async fn install_allows_required_permissions_the_actor_lacks() {
        // required_permissions restrict rather than grant, so require_superset
        // must not apply to them.
        let svc = service_with_auth_allowing(&[Permission::new(Resource::Plugin, Action::Create)]);
        let mut d = draft("acme");
        d.required_permissions = BTreeSet::from([Permission::new(Resource::Tree, Action::Delete)]);

        assert!(svc.install(Uuid::new_v4(), d).await.is_ok());
    }

    #[tokio::test]
    async fn rotate_key_replaces_the_hash() {
        let svc = service_with_unrestricted_auth();
        let (_, first) = svc.install(Uuid::nil(), draft("acme")).await.unwrap();
        let second = svc
            .rotate_key(Uuid::nil(), &PluginSlug::new("acme").unwrap())
            .await
            .unwrap();
        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn rotate_key_rejects_an_actor_lacking_the_plugins_own_permissions() {
        // Rotating returns a working credential, so it grants everything the
        // plugin may do. Without this check `plugin:update` alone would be a
        // path to deleting trees.
        let repo = repo_with_tree_deleting_plugin().await;
        let svc = service_over(
            repo,
            Some(&[Permission::new(Resource::Plugin, Action::Update)]),
        );

        let err = svc
            .rotate_key(Uuid::new_v4(), &PluginSlug::new("acme").unwrap())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Auth(_)));
    }

    #[tokio::test]
    async fn enabling_rejects_an_actor_lacking_the_plugins_own_permissions() {
        let repo = repo_with_tree_deleting_plugin().await;
        let svc = service_over(
            repo,
            Some(&[Permission::new(Resource::Plugin, Action::Update)]),
        );

        let err = svc
            .update(
                Uuid::new_v4(),
                &PluginSlug::new("acme").unwrap(),
                PluginChange {
                    enabled: Some(true),
                    ..no_change()
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Auth(_)));
    }

    #[tokio::test]
    async fn renaming_does_not_require_the_plugins_own_permissions() {
        // The counterpart to the two tests above: tightening the gate must not
        // lock an administrator out of edits that grant the plugin nothing.
        let repo = repo_with_tree_deleting_plugin().await;
        let svc = service_over(
            repo,
            Some(&[Permission::new(Resource::Plugin, Action::Update)]),
        );

        let view = svc
            .update(
                Uuid::new_v4(),
                &PluginSlug::new("acme").unwrap(),
                PluginChange {
                    name: Some(PluginName::new("Neuer Name").unwrap()),
                    ..no_change()
                },
            )
            .await
            .unwrap();
        assert_eq!(view.name.as_str(), "Neuer Name");
    }

    #[tokio::test]
    async fn disabling_does_not_require_the_plugins_own_permissions() {
        // Pulling the plug must never be harder than leaving it in.
        let repo = repo_with_tree_deleting_plugin().await;
        let svc = service_over(
            repo,
            Some(&[Permission::new(Resource::Plugin, Action::Update)]),
        );

        let view = svc
            .update(
                Uuid::new_v4(),
                &PluginSlug::new("acme").unwrap(),
                PluginChange {
                    enabled: Some(false),
                    ..no_change()
                },
            )
            .await
            .unwrap();
        assert!(!view.enabled);
    }

    use std::collections::HashMap;
    use std::sync::Mutex;

    use chrono::{DateTime, Utc};

    use domain::RepositoryError;
    use domain::authorization::OrgHierarchy;
    use domain::organization::{Organization, OrganizationReader};
    use domain::plugin::{PluginKeyHash, PluginSnapshot, TreeRef, TreeRefPage};
    use domain::role::{Role, RoleReader, RoleSnapshot};

    /// In-memory stand-in for both repository traits. Only the methods the
    /// service actually calls are implemented; the rest panic loudly so a new
    /// call site cannot pass unnoticed.
    #[derive(Default)]
    struct FakePluginRepo {
        rows: Mutex<HashMap<uuid::Uuid, Plugin>>,
    }

    #[async_trait::async_trait]
    impl PluginReader for FakePluginRepo {
        async fn by_id(&self, id: Id<Plugin>) -> Result<Plugin, RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .get(&id.value())
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }

        async fn by_slug(&self, slug: &PluginSlug) -> Result<Plugin, RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .values()
                .find(|p| p.slug() == slug)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }

        async fn all(&self) -> Result<Vec<PluginView>, RepositoryError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .values()
                .map(|p| PluginView::from_aggregate(p, None))
                .collect())
        }

        async fn view_by_slug(&self, slug: &PluginSlug) -> Result<PluginView, RepositoryError> {
            self.by_slug(slug)
                .await
                .map(|p| PluginView::from_aggregate(&p, None))
        }

        async fn last_seen_at(
            &self,
            _: Id<Plugin>,
        ) -> Result<Option<DateTime<Utc>>, RepositoryError> {
            Ok(None)
        }

        async fn tree_ref(
            &self,
            _: Id<Plugin>,
            _: &str,
        ) -> Result<Option<TreeRef>, RepositoryError> {
            unimplemented!("PluginService does not touch tree refs")
        }

        async fn tree_refs(
            &self,
            _: Id<Plugin>,
            _: Option<&str>,
            _: u32,
        ) -> Result<TreeRefPage, RepositoryError> {
            unimplemented!("PluginService does not touch tree refs")
        }
    }

    #[async_trait::async_trait]
    impl PluginWriter for FakePluginRepo {
        async fn save_new(
            &self,
            id: Id<Plugin>,
            draft: PluginDraft,
            key_hash: Option<PluginKeyHash>,
        ) -> Result<Plugin, RepositoryError> {
            let mut plugin = Plugin::reconstitute(PluginSnapshot {
                id: id.value(),
                slug: draft.slug.as_str().to_string(),
                name: draft.name.as_str().to_string(),
                description: draft.description,
                frontend_mode: "none".into(),
                frontend_target: None,
                organization_id: draft.organization_id.value(),
                permissions: draft.permissions.iter().map(|p| p.to_string()).collect(),
                required_permissions: draft
                    .required_permissions
                    .iter()
                    .map(|p| p.to_string())
                    .collect(),
                enabled: false,
                key_hash: key_hash.map(|h| h.as_str().to_string()),
                last_seen_at: None,
            });
            plugin.set_frontend(draft.frontend);
            self.rows.lock().unwrap().insert(id.value(), plugin.clone());
            Ok(plugin)
        }

        async fn save(&self, plugin: &Plugin) -> Result<(), RepositoryError> {
            self.rows
                .lock()
                .unwrap()
                .insert(plugin.id.value(), plugin.clone());
            Ok(())
        }

        async fn delete(&self, id: Id<Plugin>) -> Result<(), RepositoryError> {
            self.rows.lock().unwrap().remove(&id.value());
            Ok(())
        }

        async fn touch_last_seen(&self, _: Id<Plugin>) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn link_tree(
            &self,
            _: Id<Plugin>,
            _: &str,
            _: Id<domain::tree::Tree>,
        ) -> Result<(), RepositoryError> {
            unimplemented!("PluginService does not touch tree refs")
        }

        async fn unlink_tree(&self, _: Id<Plugin>, _: &str) -> Result<(), RepositoryError> {
            unimplemented!("PluginService does not touch tree refs")
        }
    }

    /// A single, process-wide organization id every test draft and every
    /// `service_with_auth_allowing` grant shares. `AuthorizationService`
    /// scopes permissions to a concrete organization via the real
    /// `OrgHierarchy` walk, so a fixture built before the draft exists can
    /// only line up with it by sharing a fixed id rather than a fresh
    /// `Id::new_v7()` per test.
    fn test_org() -> Id<Organization> {
        static ORG: std::sync::LazyLock<uuid::Uuid> = std::sync::LazyLock::new(uuid::Uuid::now_v7);
        Id::new(*ORG)
    }

    struct StubOrgs {
        pairs: Vec<(Id<Organization>, Option<Id<Organization>>)>,
    }

    #[async_trait::async_trait]
    impl OrganizationReader for StubOrgs {
        async fn all(&self) -> Result<Vec<Organization>, RepositoryError> {
            Ok(Vec::new())
        }
        async fn by_id(&self, _id: Id<Organization>) -> Result<Organization, RepositoryError> {
            Err(RepositoryError::NotFound)
        }
        async fn hierarchy(&self) -> Result<OrgHierarchy, RepositoryError> {
            Ok(OrgHierarchy::from_pairs(self.pairs.clone()))
        }
        async fn member_counts(&self) -> Result<HashMap<Id<Organization>, i64>, RepositoryError> {
            Ok(HashMap::new())
        }
    }

    /// Grants exactly the given permissions, for `test_org()`, to every actor.
    struct StubRoles {
        permissions: BTreeSet<Permission>,
        org: Id<Organization>,
    }

    #[async_trait::async_trait]
    impl RoleReader for StubRoles {
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
            Ok(vec![
                Role::reconstitute(RoleSnapshot {
                    id: Uuid::now_v7(),
                    organization_id: Some(self.org.value()),
                    name: "Testrolle".into(),
                    description: None,
                    permissions: self.permissions.iter().map(|p| p.to_string()).collect(),
                    template_key: None,
                })
                .unwrap(),
            ])
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

    /// Builds a service over an existing repository, so a test can install a
    /// plugin as one actor and then act on it as a less privileged one.
    /// `allowing: None` disables authorization entirely (the demo bypass).
    fn service_over(repo: Arc<FakePluginRepo>, allowing: Option<&[Permission]>) -> PluginService {
        let org = test_org();
        let authorization = Arc::new(AuthorizationService::new(
            Arc::new(StubOrgs {
                pairs: match allowing {
                    Some(_) => vec![(org, None)],
                    None => Vec::new(),
                },
            }),
            Arc::new(StubRoles {
                permissions: allowing.unwrap_or_default().iter().copied().collect(),
                org,
            }),
            allowing.is_some(),
        ));
        PluginService::new(
            repo.clone(),
            repo,
            authorization,
            Arc::new(RandomPluginKeyFactory),
        )
    }

    fn service_with_unrestricted_auth() -> PluginService {
        service_over(Arc::new(FakePluginRepo::default()), None)
    }

    fn service_with_auth_allowing(permissions: &[Permission]) -> PluginService {
        service_over(Arc::new(FakePluginRepo::default()), Some(permissions))
    }

    /// A plugin that may delete trees, installed by an unrestricted actor.
    async fn repo_with_tree_deleting_plugin() -> Arc<FakePluginRepo> {
        let repo = Arc::new(FakePluginRepo::default());
        let mut d = draft("acme");
        d.permissions = BTreeSet::from([Permission::new(Resource::Tree, Action::Delete)]);
        service_over(repo.clone(), None)
            .install(Uuid::nil(), d)
            .await
            .unwrap();
        repo
    }

    fn no_change() -> PluginChange {
        PluginChange {
            name: None,
            description: None,
            frontend: None,
            permissions: None,
            required_permissions: None,
            enabled: None,
        }
    }

    fn draft(slug: &str) -> PluginDraft {
        PluginDraft {
            slug: PluginSlug::new(slug).unwrap(),
            name: PluginName::new("Testplugin").unwrap(),
            description: None,
            organization_id: test_org(),
            permissions: BTreeSet::new(),
            required_permissions: BTreeSet::new(),
            frontend: PluginFrontend::None,
        }
    }
}
