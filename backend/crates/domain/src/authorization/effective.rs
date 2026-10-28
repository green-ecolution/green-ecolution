use std::collections::{BTreeSet, HashMap, HashSet};

use uuid::Uuid;

use crate::{Id, organization::Organization};

use super::Permission;

/// Which organizations a read path may show. `Unrestricted` skips filtering
/// entirely (demo bypass, internal non-user callers such as event handlers).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Unrestricted,
    Only(BTreeSet<Id<Organization>>),
}

impl Visibility {
    /// `None` disables filtering entirely; `Some` restricts to these orgs.
    pub fn into_raw_ids(self) -> Option<Vec<crate::RawId>> {
        match self {
            Visibility::Unrestricted => None,
            Visibility::Only(ids) => Some(ids.into_iter().map(|id| id.value()).collect()),
        }
    }

    /// For read paths that filter an already-loaded set in memory instead of
    /// pushing the org ids into a query.
    pub fn allows(&self, org: Id<Organization>) -> bool {
        match self {
            Visibility::Unrestricted => true,
            Visibility::Only(ids) => ids.contains(&org),
        }
    }
}

/// Parent map of the whole organization tree, loaded once per request.
/// Org trees have dozens of nodes, so an in-memory walk beats SQL round trips.
#[derive(Debug, Clone, Default)]
pub struct OrgHierarchy {
    parents: HashMap<Id<Organization>, Option<Id<Organization>>>,
}

impl OrgHierarchy {
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (Id<Organization>, Option<Id<Organization>>)>,
    ) -> Self {
        Self {
            parents: pairs.into_iter().collect(),
        }
    }

    /// Walks the parent chain upwards from `node`. Cycle-safe so corrupt data
    /// cannot hang a request.
    pub fn is_descendant_or_self(
        &self,
        node: Id<Organization>,
        ancestor: Id<Organization>,
    ) -> bool {
        let mut current = Some(node);
        let mut seen = HashSet::new();
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            if !seen.insert(id) {
                return false;
            }
            current = self.parents.get(&id).copied().flatten();
        }
        false
    }

    /// `parents` holds every org in the tree, so scanning its keys and testing each
    /// against `is_descendant_or_self` (itself cycle-safe) covers the whole subtree
    /// without a second, separately-maintained traversal.
    pub fn descendants_or_self(&self, ancestor: Id<Organization>) -> BTreeSet<Id<Organization>> {
        self.parents
            .keys()
            .copied()
            .filter(|node| self.is_descendant_or_self(*node, ancestor))
            .chain(std::iter::once(ancestor))
            .collect()
    }

    /// Whether the tree contains this organization at all. `parents` holds
    /// every org, so a miss means the id does not exist — the walks above
    /// cannot tell that apart from a node without a parent.
    pub fn knows(&self, node: Id<Organization>) -> bool {
        self.parents.contains_key(&node)
    }

    /// Chain from the root down to `node`, `node` last. Cycle-safe like the
    /// other walks: a repeated node ends the walk instead of hanging it.
    pub fn ancestors_from_root(&self, node: Id<Organization>) -> Vec<Id<Organization>> {
        let mut chain = Vec::new();
        let mut seen = HashSet::new();
        let mut current = Some(node);
        while let Some(id) = current {
            if !seen.insert(id) {
                break;
            }
            chain.push(id);
            current = self.parents.get(&id).copied().flatten();
        }
        chain.reverse();
        chain
    }
}

/// Union of all role grants a user holds: each grant scopes a permission set
/// to an organization subtree. Merging happens by union — "strongest scope
/// wins" falls out of checking every grant.
#[derive(Debug, Clone)]
pub struct EffectivePermissions {
    grants: Vec<(Id<Organization>, BTreeSet<Permission>)>,
    unrestricted: bool,
}

impl EffectivePermissions {
    pub fn from_grants(grants: Vec<(Id<Organization>, BTreeSet<Permission>)>) -> Self {
        Self {
            grants,
            unrestricted: false,
        }
    }

    /// Full access everywhere — only for the demo bypass (`auth.enabled = false`).
    pub fn unrestricted() -> Self {
        Self {
            grants: Vec::new(),
            unrestricted: true,
        }
    }

    pub fn allows(&self, p: Permission) -> bool {
        self.unrestricted || self.grants.iter().any(|(_, perms)| perms.contains(&p))
    }

    pub fn allows_in(&self, p: Permission, org: Id<Organization>, tree: &OrgHierarchy) -> bool {
        self.unrestricted
            || self.grants.iter().any(|(granted, perms)| {
                perms.contains(&p) && tree.is_descendant_or_self(org, *granted)
            })
    }

    pub fn superset_of(
        &self,
        required: &BTreeSet<Permission>,
        org: Id<Organization>,
        tree: &OrgHierarchy,
    ) -> bool {
        required.iter().all(|p| self.allows_in(*p, org, tree))
    }

    /// Whether at least one permission, whichever it is, reaches `org`.
    pub fn holds_any_in(&self, org: Id<Organization>, tree: &OrgHierarchy) -> bool {
        self.unrestricted
            || self.grants.iter().any(|(granted, perms)| {
                !perms.is_empty() && tree.is_descendant_or_self(org, *granted)
            })
    }

    pub fn is_unrestricted(&self) -> bool {
        self.unrestricted
    }

    pub fn grants(&self) -> &[(Id<Organization>, BTreeSet<Permission>)] {
        &self.grants
    }
}

/// The user behind a request plus the organization scope they picked, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caller {
    pub user_id: Uuid,
    pub scope: Option<Id<Organization>>,
}

impl Caller {
    pub fn unscoped(user_id: Uuid) -> Self {
        Self {
            user_id,
            scope: None,
        }
    }
}

/// Everything needed to answer authorization questions for one request.
/// A `scope` only narrows: every answer additionally has to lie inside the
/// scope's subtree, it never grants anything.
#[derive(Debug, Clone)]
pub struct AccessContext {
    pub permissions: EffectivePermissions,
    pub hierarchy: OrgHierarchy,
    pub scope: Option<Id<Organization>>,
}

impl AccessContext {
    pub fn unrestricted() -> Self {
        Self {
            permissions: EffectivePermissions::unrestricted(),
            hierarchy: OrgHierarchy::default(),
            scope: None,
        }
    }

    pub fn with_scope(mut self, scope: Id<Organization>) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn without_scope(&self) -> Self {
        Self {
            scope: None,
            ..self.clone()
        }
    }

    fn in_scope(&self, org: Id<Organization>) -> bool {
        self.scope
            .is_none_or(|scope| self.hierarchy.is_descendant_or_self(org, scope))
    }

    fn narrow(&self, visibility: Visibility) -> Visibility {
        let Some(scope) = self.scope else {
            return visibility;
        };
        let subtree = self.hierarchy.descendants_or_self(scope);
        match visibility {
            Visibility::Unrestricted => Visibility::Only(subtree),
            Visibility::Only(orgs) => {
                Visibility::Only(orgs.intersection(&subtree).copied().collect())
            }
        }
    }

    pub fn allows_in(&self, p: Permission, org: Id<Organization>) -> bool {
        self.in_scope(org) && self.permissions.allows_in(p, org, &self.hierarchy)
    }

    pub fn superset_of(&self, required: &BTreeSet<Permission>, org: Id<Organization>) -> bool {
        self.in_scope(org) && self.permissions.superset_of(required, org, &self.hierarchy)
    }

    pub fn holds_any_in(&self, org: Id<Organization>) -> bool {
        self.in_scope(org) && self.permissions.holds_any_in(org, &self.hierarchy)
    }

    /// Whether `p` is granted anywhere inside the scope (anywhere at all
    /// without one).
    pub fn allows_anywhere(&self, p: Permission) -> bool {
        match self.visible_orgs(p) {
            Visibility::Unrestricted => true,
            Visibility::Only(orgs) => !orgs.is_empty(),
        }
    }

    /// Whether the user may pick `scope`: it lies in their reachable
    /// organizations, ignoring any scope already set on this context.
    pub fn admits_scope(&self, scope: Id<Organization>) -> bool {
        match self.without_scope().reachable_orgs() {
            Visibility::Unrestricted => self.hierarchy.knows(scope),
            Visibility::Only(orgs) => orgs.contains(&scope),
        }
    }

    pub fn visible_orgs(&self, p: Permission) -> Visibility {
        if self.permissions.is_unrestricted() {
            return self.narrow(Visibility::Unrestricted);
        }
        let mut orgs = BTreeSet::new();
        for (granted, perms) in self.permissions.grants() {
            if perms.contains(&p) {
                orgs.extend(self.hierarchy.descendants_or_self(*granted));
            }
        }
        self.narrow(Visibility::Only(orgs))
    }

    /// Organizations any of the user's grants reaches, whatever the permission.
    /// Answers "where does this user work", not "what may they do there".
    pub fn reachable_orgs(&self) -> Visibility {
        if self.permissions.is_unrestricted() {
            return self.narrow(Visibility::Unrestricted);
        }
        let mut orgs = BTreeSet::new();
        for (granted, perms) in self.permissions.grants() {
            if !perms.is_empty() {
                orgs.extend(self.hierarchy.descendants_or_self(*granted));
            }
        }
        self.narrow(Visibility::Only(orgs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{Action, Permission, ResourceKey};
    use crate::{Id, organization::Organization};
    use std::collections::BTreeSet;

    const WIDGET: ResourceKey = ResourceKey::declare("widget");

    fn ids() -> (Id<Organization>, Id<Organization>, Id<Organization>) {
        (Id::new_v7(), Id::new_v7(), Id::new_v7())
    }

    fn tree_read() -> Permission {
        Permission::new(WIDGET, Action::Read)
    }

    #[test]
    fn descendant_or_self_walks_up_the_chain() {
        let (root, tbz, sub) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root)), (sub, Some(tbz))]);
        assert!(h.is_descendant_or_self(sub, sub));
        assert!(h.is_descendant_or_self(sub, tbz));
        assert!(h.is_descendant_or_self(sub, root));
        assert!(!h.is_descendant_or_self(tbz, sub));
    }

    #[test]
    fn descendant_check_is_cycle_safe() {
        let (a, b, _) = ids();
        // Defensive: real data cannot cycle (DB enforces a single root and no
        // moves exist), but the walk must still terminate on corrupt input.
        let h = OrgHierarchy::from_pairs([(a, Some(b)), (b, Some(a))]);
        let (_, _, other) = ids();
        assert!(!h.is_descendant_or_self(a, other));
    }

    #[test]
    fn allows_in_grants_subtree_never_upwards() {
        let (root, tbz, sub) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root)), (sub, Some(tbz))]);
        let eff = EffectivePermissions::from_grants(vec![(tbz, BTreeSet::from([tree_read()]))]);
        assert!(eff.allows_in(tree_read(), tbz, &h));
        assert!(eff.allows_in(tree_read(), sub, &h));
        assert!(!eff.allows_in(tree_read(), root, &h));
    }

    #[test]
    fn allows_merges_multiple_grants() {
        let (_, tbz, sub) = ids();
        let write = Permission::new(WIDGET, Action::Update);
        let eff = EffectivePermissions::from_grants(vec![
            (tbz, BTreeSet::from([tree_read()])),
            (sub, BTreeSet::from([write])),
        ]);
        assert!(eff.allows(tree_read()));
        assert!(eff.allows(write));
        assert!(!eff.allows(Permission::new(WIDGET, Action::Delete)));
    }

    #[test]
    fn superset_of_requires_every_permission_in_scope() {
        let (root, tbz, _) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root))]);
        let write = Permission::new(WIDGET, Action::Update);
        let eff = EffectivePermissions::from_grants(vec![(tbz, BTreeSet::from([tree_read()]))]);
        assert!(eff.superset_of(&BTreeSet::from([tree_read()]), tbz, &h));
        assert!(!eff.superset_of(&BTreeSet::from([tree_read(), write]), tbz, &h));
    }

    #[test]
    fn holds_any_in_follows_the_subtree_and_skips_empty_grants() {
        let (root, tbz, sub) = ids();
        let sibling = Id::new_v7();
        let h = OrgHierarchy::from_pairs([
            (root, None),
            (tbz, Some(root)),
            (sub, Some(tbz)),
            (sibling, Some(root)),
        ]);
        let eff = EffectivePermissions::from_grants(vec![
            (tbz, BTreeSet::from([tree_read()])),
            (sibling, BTreeSet::new()),
        ]);
        assert!(eff.holds_any_in(tbz, &h));
        assert!(eff.holds_any_in(sub, &h));
        assert!(!eff.holds_any_in(root, &h));
        assert!(!eff.holds_any_in(sibling, &h));
        assert!(EffectivePermissions::unrestricted().holds_any_in(root, &h));
    }

    #[test]
    fn unrestricted_allows_everything_everywhere() {
        let (root, ..) = ids();
        let ctx = AccessContext::unrestricted();
        assert!(ctx.allows_in(tree_read(), root));
        assert!(
            ctx.superset_of(
                &BTreeSet::from_iter(
                    crate::authorization::ResourceCatalog::new([WIDGET])
                        .unwrap()
                        .permissions()
                ),
                root
            )
        );
    }

    #[test]
    fn knows_separates_a_missing_org_from_a_parentless_one() {
        let (root, tbz, unknown) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root))]);
        assert!(h.knows(root));
        assert!(h.knows(tbz));
        assert!(!h.knows(unknown));
    }

    #[test]
    fn ancestors_from_root_returns_the_chain_root_first() {
        let (root, tbz, sub) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root)), (sub, Some(tbz))]);
        assert_eq!(h.ancestors_from_root(sub), vec![root, tbz, sub]);
        assert_eq!(h.ancestors_from_root(root), vec![root]);
    }

    #[test]
    fn ancestors_from_root_is_cycle_safe() {
        let (a, b, _) = ids();
        let h = OrgHierarchy::from_pairs([(a, Some(b)), (b, Some(a))]);
        let chain = h.ancestors_from_root(a);
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn ancestors_from_root_of_an_unknown_node_is_just_itself() {
        let (root, _, unknown) = ids();
        let h = OrgHierarchy::from_pairs([(root, None)]);
        assert_eq!(h.ancestors_from_root(unknown), vec![unknown]);
    }

    #[test]
    fn visible_orgs_unions_subtrees_never_upwards() {
        let (root, tbz, sub) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root)), (sub, Some(tbz))]);
        let eff = EffectivePermissions::from_grants(vec![(tbz, BTreeSet::from([tree_read()]))]);
        let ctx = AccessContext {
            permissions: eff,
            hierarchy: h,
            scope: None,
        };
        match ctx.visible_orgs(tree_read()) {
            Visibility::Only(orgs) => {
                assert!(orgs.contains(&tbz) && orgs.contains(&sub));
                assert!(!orgs.contains(&root));
            }
            Visibility::Unrestricted => panic!("expected Only"),
        }
        // permission not granted anywhere -> empty set, not unrestricted
        let del = Permission::new(WIDGET, Action::Delete);
        assert_eq!(ctx.visible_orgs(del), Visibility::Only(BTreeSet::new()));
    }

    #[test]
    fn allows_passes_everything_when_unrestricted_and_filters_otherwise() {
        let (root, tbz, _) = ids();
        assert!(Visibility::Unrestricted.allows(root));
        let only = Visibility::Only(BTreeSet::from([tbz]));
        assert!(only.allows(tbz));
        assert!(!only.allows(root));
    }

    #[test]
    fn reachable_orgs_covers_every_subtree_with_any_grant() {
        let (root, tbz, sub) = ids();
        let other = Id::new_v7();
        let hierarchy = OrgHierarchy::from_pairs([
            (root, None),
            (tbz, Some(root)),
            (sub, Some(tbz)),
            (other, Some(root)),
        ]);
        let ctx = AccessContext {
            permissions: EffectivePermissions::from_grants(vec![(
                tbz,
                BTreeSet::from([tree_read()]),
            )]),
            hierarchy,
            scope: None,
        };
        assert_eq!(
            ctx.reachable_orgs(),
            Visibility::Only(BTreeSet::from([tbz, sub]))
        );
    }

    #[test]
    fn reachable_orgs_ignores_empty_grants_and_is_unrestricted_for_the_bypass() {
        let (root, tbz, _) = ids();
        let hierarchy = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root))]);
        let ctx = AccessContext {
            permissions: EffectivePermissions::from_grants(vec![(tbz, BTreeSet::new())]),
            hierarchy,
            scope: None,
        };
        assert_eq!(ctx.reachable_orgs(), Visibility::Only(BTreeSet::new()));
        assert_eq!(
            AccessContext::unrestricted().reachable_orgs(),
            Visibility::Unrestricted
        );
    }

    #[test]
    fn visible_orgs_unrestricted_for_demo_context() {
        assert_eq!(
            AccessContext::unrestricted().visible_orgs(tree_read()),
            Visibility::Unrestricted
        );
    }

    struct ScopeTree {
        root: Id<Organization>,
        tbz: Id<Organization>,
        firma_a: Id<Organization>,
        firma_b: Id<Organization>,
        firma_a1: Id<Organization>,
        hierarchy: OrgHierarchy,
    }

    fn scope_tree() -> ScopeTree {
        let (root, tbz, firma_a) = ids();
        let (firma_b, firma_a1) = (Id::new_v7(), Id::new_v7());
        let hierarchy = OrgHierarchy::from_pairs([
            (root, None),
            (tbz, Some(root)),
            (firma_a, Some(tbz)),
            (firma_b, Some(tbz)),
            (firma_a1, Some(firma_a)),
        ]);
        ScopeTree {
            root,
            tbz,
            firma_a,
            firma_b,
            firma_a1,
            hierarchy,
        }
    }

    fn ctx_with_grant(t: &ScopeTree, at: Id<Organization>) -> AccessContext {
        AccessContext {
            permissions: EffectivePermissions::from_grants(vec![(
                at,
                BTreeSet::from([tree_read()]),
            )]),
            hierarchy: t.hierarchy.clone(),
            scope: None,
        }
    }

    #[test]
    fn scope_narrows_ancestor_grant() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.tbz).with_scope(t.firma_a);
        assert!(ctx.allows_in(tree_read(), t.firma_a1));
        assert!(!ctx.allows_in(tree_read(), t.tbz));
        assert!(!ctx.allows_in(tree_read(), t.firma_b));
        assert_eq!(
            ctx.visible_orgs(tree_read()),
            Visibility::Only(BTreeSet::from([t.firma_a, t.firma_a1]))
        );
    }

    #[test]
    fn scope_keeps_descendant_grant() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.firma_a1).with_scope(t.tbz);
        assert!(ctx.allows_in(tree_read(), t.firma_a1));
    }

    #[test]
    fn scope_drops_foreign_grant() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.firma_b).with_scope(t.firma_a);
        assert_eq!(
            ctx.visible_orgs(tree_read()),
            Visibility::Only(BTreeSet::new())
        );
        assert!(!ctx.allows_anywhere(tree_read()));
    }

    #[test]
    fn scope_narrows_unrestricted() {
        let t = scope_tree();
        let ctx = AccessContext {
            permissions: EffectivePermissions::unrestricted(),
            hierarchy: t.hierarchy.clone(),
            scope: Some(t.firma_a),
        };
        assert_eq!(
            ctx.visible_orgs(tree_read()),
            Visibility::Only(BTreeSet::from([t.firma_a, t.firma_a1]))
        );
        assert!(!ctx.allows_in(tree_read(), t.tbz));
        assert!(ctx.allows_in(tree_read(), t.firma_a1));
    }

    #[test]
    fn scope_narrows_superset_and_holds_any() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.tbz).with_scope(t.firma_a);
        let required = BTreeSet::from([tree_read()]);
        assert!(ctx.superset_of(&required, t.firma_a1));
        assert!(!ctx.superset_of(&required, t.firma_b));
        assert!(ctx.holds_any_in(t.firma_a));
        assert!(!ctx.holds_any_in(t.tbz));
        assert_eq!(
            ctx.reachable_orgs(),
            Visibility::Only(BTreeSet::from([t.firma_a, t.firma_a1]))
        );
    }

    #[test]
    fn without_scope_restores_full_view() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.tbz).with_scope(t.firma_a);
        assert!(ctx.without_scope().allows_in(tree_read(), t.tbz));
        assert_eq!(ctx.without_scope().scope, None);
    }

    #[test]
    fn admits_scope_only_reachable() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.firma_a);
        assert!(ctx.admits_scope(t.firma_a1));
        assert!(!ctx.admits_scope(t.tbz));

        let unrestricted = AccessContext {
            permissions: EffectivePermissions::unrestricted(),
            hierarchy: t.hierarchy.clone(),
            scope: None,
        };
        assert!(unrestricted.admits_scope(t.root));
        assert!(!unrestricted.admits_scope(Id::new_v7()));
    }

    #[test]
    fn allows_anywhere_without_scope_matches_allows() {
        let t = scope_tree();
        let ctx = ctx_with_grant(&t, t.firma_a);
        assert!(ctx.allows_anywhere(tree_read()));
        assert!(!ctx.allows_anywhere(Permission::new(WIDGET, Action::Delete)));
        assert!(AccessContext::unrestricted().allows_anywhere(tree_read()));
    }
}
