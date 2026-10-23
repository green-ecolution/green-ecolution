use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    deps: BTreeMap<String, BTreeSet<String>>,
    order: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("unit `{unit}` depends on unknown unit `{dependency}`")]
    UnknownDependency { unit: String, dependency: String },
    #[error("dependency cycle between {0}")]
    Cycle(String),
    #[error("unknown unit `{0}`")]
    UnknownUnit(String),
}

impl Graph {
    pub fn new<I, D>(edges: I) -> Result<Self, GraphError>
    where
        I: IntoIterator<Item = (String, D)>,
        D: IntoIterator<Item = String>,
    {
        let deps: BTreeMap<String, BTreeSet<String>> = edges
            .into_iter()
            .map(|(unit, deps)| (unit, deps.into_iter().collect()))
            .collect();
        for (unit, unit_deps) in &deps {
            if let Some(missing) = unit_deps.iter().find(|d| !deps.contains_key(*d)) {
                return Err(GraphError::UnknownDependency {
                    unit: unit.clone(),
                    dependency: missing.clone(),
                });
            }
        }
        let order = topological(&deps)?;
        Ok(Self { deps, order })
    }

    pub fn contains(&self, unit: &str) -> bool {
        self.deps.contains_key(unit)
    }

    pub fn start_order(&self) -> &[String] {
        &self.order
    }

    pub fn dependencies(&self, unit: &str) -> impl Iterator<Item = &str> {
        self.deps
            .get(unit)
            .into_iter()
            .flatten()
            .map(String::as_str)
    }

    pub fn transitive_dependents(&self, unit: &str) -> Vec<String> {
        let mut affected = BTreeSet::from([unit]);
        for candidate in &self.order {
            if self.dependencies(candidate).any(|d| affected.contains(d)) {
                affected.insert(candidate.as_str());
            }
        }
        self.order
            .iter()
            .filter(|candidate| candidate.as_str() != unit && affected.contains(candidate.as_str()))
            .cloned()
            .collect()
    }

    pub fn closure(&self, roots: &[String]) -> Result<BTreeSet<String>, GraphError> {
        let mut stack = Vec::with_capacity(roots.len());
        for root in roots {
            if !self.contains(root) {
                return Err(GraphError::UnknownUnit(root.clone()));
            }
            stack.push(root.as_str());
        }
        let mut seen = BTreeSet::new();
        while let Some(unit) = stack.pop() {
            if seen.insert(unit.to_owned()) {
                stack.extend(self.dependencies(unit));
            }
        }
        Ok(seen)
    }

    pub fn subgraph(&self, keep: &BTreeSet<String>) -> Graph {
        let deps = self
            .deps
            .iter()
            .filter(|(unit, _)| keep.contains(*unit))
            .map(|(unit, unit_deps)| {
                let kept = unit_deps
                    .iter()
                    .filter(|d| keep.contains(*d))
                    .cloned()
                    .collect();
                (unit.clone(), kept)
            })
            .collect();
        let order = self
            .order
            .iter()
            .filter(|u| keep.contains(*u))
            .cloned()
            .collect();
        Graph { deps, order }
    }
}

/// Kahn's algorithm over sorted maps, so equal inputs always give the same order.
fn topological(deps: &BTreeMap<String, BTreeSet<String>>) -> Result<Vec<String>, GraphError> {
    let mut remaining: BTreeMap<&str, BTreeSet<&str>> = deps
        .iter()
        .map(|(unit, d)| (unit.as_str(), d.iter().map(String::as_str).collect()))
        .collect();
    let mut order = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let ready: Vec<&str> = remaining
            .iter()
            .filter(|(_, d)| d.is_empty())
            .map(|(unit, _)| *unit)
            .collect();
        if ready.is_empty() {
            let stuck: Vec<&str> = remaining.keys().copied().collect();
            return Err(GraphError::Cycle(stuck.join(", ")));
        }
        for unit in &ready {
            remaining.remove(unit);
        }
        for unit_deps in remaining.values_mut() {
            for unit in &ready {
                unit_deps.remove(unit);
            }
        }
        order.extend(ready.into_iter().map(str::to_owned));
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(edges: &[(&str, &[&str])]) -> Result<Graph, GraphError> {
        Graph::new(edges.iter().map(|(unit, deps)| {
            (
                (*unit).to_owned(),
                deps.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>(),
            )
        }))
    }

    fn stack() -> Graph {
        graph(&[
            ("backend", &["migrations", "keycloak"]),
            ("frontend", &[]),
            ("migrations", &["db"]),
            ("keycloak", &["keycloak-init-db"]),
            ("keycloak-init-db", &["db"]),
            ("db", &[]),
        ])
        .unwrap()
    }

    #[test]
    fn orders_dependencies_first_and_deterministically() {
        assert_eq!(
            stack().start_order(),
            [
                "db",
                "frontend",
                "keycloak-init-db",
                "migrations",
                "keycloak",
                "backend"
            ]
        );
    }

    #[test]
    fn rejects_unknown_dependencies() {
        assert_eq!(
            graph(&[("backend", &["db"])]),
            Err(GraphError::UnknownDependency {
                unit: "backend".into(),
                dependency: "db".into()
            })
        );
    }

    #[test]
    fn rejects_cycles() {
        assert!(matches!(
            graph(&[("a", &["b"]), ("b", &["a"]), ("c", &[])]),
            Err(GraphError::Cycle(units)) if units == "a, b"
        ));
    }

    #[test]
    fn lists_transitive_dependents_in_start_order() {
        assert_eq!(
            stack().transitive_dependents("db"),
            ["keycloak-init-db", "migrations", "keycloak", "backend"]
        );
        assert!(stack().transitive_dependents("frontend").is_empty());
    }

    #[test]
    fn closure_pulls_in_every_dependency() {
        let selected = stack().closure(&["backend".into()]).unwrap();
        assert_eq!(
            selected.into_iter().collect::<Vec<_>>(),
            [
                "backend",
                "db",
                "keycloak",
                "keycloak-init-db",
                "migrations"
            ]
        );
        assert_eq!(
            stack().closure(&["nope".into()]),
            Err(GraphError::UnknownUnit("nope".into()))
        );
    }

    #[test]
    fn subgraph_keeps_order_and_drops_foreign_edges() {
        let keep = BTreeSet::from(["db".to_owned(), "migrations".to_owned()]);
        let sub = stack().subgraph(&keep);
        assert_eq!(sub.start_order(), ["db", "migrations"]);
        assert!(!sub.contains("backend"));
        assert_eq!(sub.dependencies("migrations").collect::<Vec<_>>(), ["db"]);
    }
}
