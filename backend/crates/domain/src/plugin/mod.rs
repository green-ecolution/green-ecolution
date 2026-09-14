//! Plugin aggregate — an external system installed by an administrator that
//! may write into the domain and may contribute a view to the frontend.
//!
//! `slug` is private and immutable: it doubles as the `ProviderId` stamped on
//! every record the plugin imports, so renaming it would orphan that data.
//!
//! Two permission sets that look alike but answer different questions:
//! `permissions` is what the plugin itself may do, `required_permissions` is
//! what a user needs in order to open its view.

pub mod error;
pub mod repository;
pub mod snapshot;
pub mod view;

use std::collections::BTreeSet;

use url::Url;

use crate::{
    Id, authorization::Permission, organization::Organization, shared::error::ValidationError,
};

pub use error::PluginError;
pub use repository::{PluginReader, PluginWriter, TreeRef, TreeRefPage};
#[doc(hidden)]
pub use snapshot::PluginSnapshot;
pub use view::PluginView;

crate::newtype_nonempty! {
    /// Human-readable plugin name, 1–255 characters after trimming.
    PluginName, "plugin.name", 1, 255
}

crate::newtype_nonempty! {
    /// Hash of the plugin's API key. The domain never sees the plaintext.
    PluginKeyHash, "plugin.key_hash", 1, 255
}

/// Slugs the plugin routes already spend on static segments. A plugin
/// carrying one of these is reachable by no admin endpoint afterwards,
/// because `/plugins/<segment>` matches the static route before the dynamic
/// `/plugins/{plugin_slug}` — the row could then only be removed with direct
/// database access, while still pinning its organization against deletion.
/// Only single-segment collisions belong here: `ingest` is safe because the
/// route below it is `/plugins/ingest/trees`, one level deeper than any
/// admin endpoint.
pub const RESERVED_PLUGIN_SLUGS: &[&str] = &["me"];

/// Immutable identifier, also used as the `ProviderId` on imported records.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginSlug(String);

impl PluginSlug {
    pub fn new(raw: impl Into<String>) -> Result<Self, ValidationError> {
        let raw: String = raw.into();
        let trimmed = raw.trim();
        let valid = !trimmed.is_empty()
            && trimmed.len() <= 64
            && trimmed
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !trimmed.starts_with('-')
            && !trimmed.ends_with('-');
        if !valid {
            return Err(ValidationError::InvalidFormat {
                field: "plugin.slug",
                reason: "expected 1-64 chars of [a-z0-9-], not starting or ending with '-'".into(),
            });
        }
        if RESERVED_PLUGIN_SLUGS.contains(&trimmed) {
            return Err(ValidationError::InvalidFormat {
                field: "plugin.slug",
                reason: format!(
                    "'{trimmed}' is reserved by the plugin api and cannot be used as a slug"
                ),
            });
        }
        Ok(Self(trimmed.to_string()))
    }

    #[doc(hidden)]
    pub fn reconstitute(raw: String) -> Self {
        Self(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Host and port of an in-cluster service. Whether the host is allowed is a
/// deployment question answered by the service layer against the configured
/// allowlist, not by the domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEndpoint {
    host: String,
    port: u16,
}

impl ServiceEndpoint {
    pub fn new(host: impl Into<String>, port: u16) -> Result<Self, ValidationError> {
        let host: String = host.into();
        let host = host.trim().to_string();
        if host.is_empty() || host.len() > 253 || host.contains('/') {
            return Err(ValidationError::InvalidFormat {
                field: "plugin.frontend_target",
                reason: "expected a bare host name without a path".into(),
            });
        }
        Ok(Self { host, port })
    }

    #[doc(hidden)]
    pub fn reconstitute(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginFrontend {
    /// No view, a pure data provider.
    None,
    /// Publicly hosted by the operator.
    External(Url),
    /// Runs in the cluster, proxied by Green Ecolution.
    Proxied(ServiceEndpoint),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    pub id: Id<Plugin>,
    pub name: PluginName,
    pub description: Option<String>,

    slug: PluginSlug,
    organization_id: Id<Organization>,
    permissions: BTreeSet<Permission>,
    required_permissions: BTreeSet<Permission>,
    frontend: PluginFrontend,
    enabled: bool,
    key_hash: Option<PluginKeyHash>,
}

/// Input for installing a new [`Plugin`]. New plugins start disabled and
/// without a key; both are separate, explicit steps.
#[derive(Debug, Clone)]
pub struct PluginDraft {
    pub slug: PluginSlug,
    pub name: PluginName,
    pub description: Option<String>,
    pub organization_id: Id<Organization>,
    pub permissions: BTreeSet<Permission>,
    pub required_permissions: BTreeSet<Permission>,
    pub frontend: PluginFrontend,
}

impl Plugin {
    #[doc(hidden)]
    pub fn reconstitute(snap: PluginSnapshot) -> Self {
        let frontend = match snap.frontend_mode.as_str() {
            "external" => PluginFrontend::External(
                snap.frontend_target
                    .as_deref()
                    .and_then(|t| Url::parse(t).ok())
                    .expect("persisted external plugin must carry a valid url"),
            ),
            "proxied" => {
                let target = snap
                    .frontend_target
                    .as_deref()
                    .expect("persisted proxied plugin must carry a target");
                let (host, port) = target
                    .rsplit_once(':')
                    .expect("persisted proxy target must be host:port");
                PluginFrontend::Proxied(ServiceEndpoint::reconstitute(
                    host.to_string(),
                    port.parse().expect("persisted proxy port must be numeric"),
                ))
            }
            _ => PluginFrontend::None,
        };

        Self {
            id: Id::new(snap.id),
            name: PluginName::reconstitute(snap.name),
            description: snap.description,
            slug: PluginSlug::reconstitute(snap.slug),
            organization_id: Id::new(snap.organization_id),
            permissions: parse_persisted(&snap.permissions),
            required_permissions: parse_persisted(&snap.required_permissions),
            frontend,
            enabled: snap.enabled,
            key_hash: snap.key_hash.map(PluginKeyHash::reconstitute),
        }
    }

    pub fn slug(&self) -> &PluginSlug {
        &self.slug
    }

    pub fn organization_id(&self) -> Id<Organization> {
        self.organization_id
    }

    pub fn permissions(&self) -> &BTreeSet<Permission> {
        &self.permissions
    }

    pub fn required_permissions(&self) -> &BTreeSet<Permission> {
        &self.required_permissions
    }

    pub fn frontend(&self) -> &PluginFrontend {
        &self.frontend
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn key_hash(&self) -> Option<&PluginKeyHash> {
        self.key_hash.as_ref()
    }

    pub fn rename(&mut self, new_name: PluginName) {
        if self.name == new_name {
            return;
        }
        self.name = new_name;
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub fn set_frontend(&mut self, frontend: PluginFrontend) {
        self.frontend = frontend;
    }

    pub fn replace_permissions(&mut self, permissions: BTreeSet<Permission>) {
        self.permissions = permissions;
    }

    pub fn replace_required_permissions(&mut self, permissions: BTreeSet<Permission>) {
        self.required_permissions = permissions;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn rotate_credential(&mut self, hash: PluginKeyHash) {
        self.key_hash = Some(hash);
    }
}

/// Permissions come out of a `TEXT[]` column the schema does not constrain.
/// An unparsable entry is dropped rather than panicking: it can only mean the
/// catalog shrank, and losing a grant is safer than refusing to load the row.
fn parse_persisted(raw: &[String]) -> BTreeSet<Permission> {
    raw.iter().filter_map(|p| p.parse().ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{Action, Resource};
    use claims::{assert_err, assert_ok};

    fn fixed() -> Plugin {
        Plugin {
            id: Id::new_v7(),
            name: PluginName::new("Baumkataster").unwrap(),
            description: None,
            slug: PluginSlug::new("tbz-baumkataster").unwrap(),
            organization_id: Id::new_v7(),
            permissions: BTreeSet::from([Permission::new(Resource::Tree, Action::Create)]),
            required_permissions: BTreeSet::new(),
            frontend: PluginFrontend::None,
            enabled: false,
            key_hash: None,
        }
    }

    #[test]
    fn slug_rejects_empty() {
        assert_err!(PluginSlug::new(""));
    }

    #[test]
    fn slug_rejects_uppercase_and_spaces() {
        assert_err!(PluginSlug::new("TBZ Kataster"));
    }

    #[test]
    fn slug_rejects_leading_dash() {
        assert_err!(PluginSlug::new("-kataster"));
    }

    #[test]
    fn slug_accepts_valid() {
        assert_ok!(PluginSlug::new("tbz-baumkataster"));
    }

    /// `/plugins/me` is the ingest self-lookup route and wins over the dynamic
    /// `/plugins/{plugin_slug}` in the router, so a plugin installed under
    /// this slug could never be read, renamed, disabled or uninstalled again.
    #[test]
    fn slug_rejects_a_reserved_route_segment() {
        for reserved in RESERVED_PLUGIN_SLUGS {
            assert_err!(PluginSlug::new(*reserved));
            assert_err!(PluginSlug::new(format!("  {reserved} ")));
        }
    }

    /// The reservation covers the exact segment only; a longer slug that
    /// merely starts with it still routes to the dynamic segment.
    #[test]
    fn slug_accepts_a_reserved_segment_as_a_prefix() {
        assert_ok!(PluginSlug::new("me-too"));
        assert_ok!(PluginSlug::new("ingest"));
    }

    #[test]
    fn service_endpoint_rejects_path() {
        assert_err!(ServiceEndpoint::new("svc.local/ui", 8080));
    }

    #[test]
    fn rename_to_same_is_noop() {
        let mut p = fixed();
        p.rename(PluginName::new("Baumkataster").unwrap());
        assert_eq!(p.name.as_str(), "Baumkataster");
    }

    #[test]
    fn rename_changes_name() {
        let mut p = fixed();
        p.rename(PluginName::new("Kataster Flensburg").unwrap());
        assert_eq!(p.name.as_str(), "Kataster Flensburg");
    }

    #[test]
    fn enable_and_disable_toggle() {
        let mut p = fixed();
        assert!(!p.enabled());
        p.enable();
        assert!(p.enabled());
        p.disable();
        assert!(!p.enabled());
    }

    #[test]
    fn new_plugin_has_no_credential() {
        assert!(fixed().key_hash().is_none());
    }

    #[test]
    fn rotate_credential_sets_hash() {
        let mut p = fixed();
        p.rotate_credential(PluginKeyHash::new("abc123").unwrap());
        assert_eq!(p.key_hash().unwrap().as_str(), "abc123");
    }

    #[test]
    fn replace_permissions_swaps_the_set() {
        let mut p = fixed();
        let target = BTreeSet::from([Permission::new(Resource::Tree, Action::Delete)]);
        p.replace_permissions(target.clone());
        assert_eq!(p.permissions(), &target);
    }

    #[test]
    fn required_permissions_are_independent_of_permissions() {
        let mut p = fixed();
        p.replace_required_permissions(BTreeSet::from([Permission::new(
            Resource::Tree,
            Action::Read,
        )]));
        assert!(
            p.permissions()
                .contains(&Permission::new(Resource::Tree, Action::Create))
        );
        assert!(
            !p.required_permissions()
                .contains(&Permission::new(Resource::Tree, Action::Create))
        );
    }

    #[test]
    fn reconstitute_drops_unknown_permission_strings() {
        let mut snap = snapshot_of(&fixed());
        snap.permissions = vec!["tree:read".into(), "does_not_exist:read".into()];
        let p = Plugin::reconstitute(snap);
        assert_eq!(p.permissions().len(), 1);
    }

    fn snapshot_of(p: &Plugin) -> PluginSnapshot {
        PluginSnapshot {
            id: p.id.value(),
            slug: p.slug.as_str().to_string(),
            name: p.name.as_str().to_string(),
            description: p.description.clone(),
            frontend_mode: "none".into(),
            frontend_target: None,
            organization_id: p.organization_id.value(),
            permissions: vec![],
            required_permissions: vec![],
            enabled: p.enabled,
            key_hash: None,
            last_seen_at: None,
        }
    }
}
