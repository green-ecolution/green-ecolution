use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{Id, organization::Organization};

use super::Permission;

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
}

/// Everything needed to answer authorization questions for one request.
#[derive(Debug, Clone)]
pub struct AccessContext {
    pub permissions: EffectivePermissions,
    pub hierarchy: OrgHierarchy,
}

impl AccessContext {
    pub fn unrestricted() -> Self {
        Self {
            permissions: EffectivePermissions::unrestricted(),
            hierarchy: OrgHierarchy::default(),
        }
    }

    pub fn allows_in(&self, p: Permission, org: Id<Organization>) -> bool {
        self.permissions.allows_in(p, org, &self.hierarchy)
    }

    pub fn superset_of(&self, required: &BTreeSet<Permission>, org: Id<Organization>) -> bool {
        self.permissions.superset_of(required, org, &self.hierarchy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::{Action, Permission, Resource};
    use crate::{Id, organization::Organization};
    use std::collections::BTreeSet;

    fn ids() -> (Id<Organization>, Id<Organization>, Id<Organization>) {
        (Id::new_v7(), Id::new_v7(), Id::new_v7())
    }

    fn tree_read() -> Permission {
        Permission::new(Resource::Tree, Action::Read)
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
        let write = Permission::new(Resource::Tree, Action::Update);
        let eff = EffectivePermissions::from_grants(vec![
            (tbz, BTreeSet::from([tree_read()])),
            (sub, BTreeSet::from([write])),
        ]);
        assert!(eff.allows(tree_read()));
        assert!(eff.allows(write));
        assert!(!eff.allows(Permission::new(Resource::Tree, Action::Delete)));
    }

    #[test]
    fn superset_of_requires_every_permission_in_scope() {
        let (root, tbz, _) = ids();
        let h = OrgHierarchy::from_pairs([(root, None), (tbz, Some(root))]);
        let write = Permission::new(Resource::Tree, Action::Update);
        let eff = EffectivePermissions::from_grants(vec![(tbz, BTreeSet::from([tree_read()]))]);
        assert!(eff.superset_of(&BTreeSet::from([tree_read()]), tbz, &h));
        assert!(!eff.superset_of(&BTreeSet::from([tree_read(), write]), tbz, &h));
    }

    #[test]
    fn unrestricted_allows_everything_everywhere() {
        let (root, ..) = ids();
        let ctx = AccessContext::unrestricted();
        assert!(ctx.allows_in(tree_read(), root));
        assert!(ctx.superset_of(&BTreeSet::from_iter(Permission::catalog()), root));
    }
}
