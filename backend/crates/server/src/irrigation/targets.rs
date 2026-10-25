//! Trees and clusters as targets the core can point at.

use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use domain::{
    Id, RepositoryError,
    organization::Organization,
    shared::coordinates::Coordinate,
    target::{TargetCatalog, TargetError, TargetKindSpec, TargetPurpose, TargetRef},
};
use irrigation_domain::{
    cluster::{TreeCluster, TreeClusterReader},
    tree::{Tree, TreeReader},
};

use super::{KEY, resources};

pub const TREE_KIND: &str = "tree";
pub const CLUSTER_KIND: &str = "cluster";

const KINDS: [TargetKindSpec; 2] = [
    TargetKindSpec {
        kind: TREE_KIND,
        accepts_sensors: true,
        commentable: false,
        resource: resources::TREE,
    },
    TargetKindSpec {
        kind: CLUSTER_KIND,
        accepts_sensors: false,
        commentable: true,
        resource: resources::TREE_CLUSTER,
    },
];

pub fn tree_target(id: Id<Tree>) -> TargetRef {
    TargetRef::new(KEY, TREE_KIND, id.value()).expect("static keys are valid")
}

pub fn cluster_target(id: Id<TreeCluster>) -> TargetRef {
    TargetRef::new(KEY, CLUSTER_KIND, id.value()).expect("static keys are valid")
}

pub fn is_tree(target: &TargetRef) -> bool {
    target.module.as_str() == KEY && target.kind.as_str() == TREE_KIND
}

fn is_cluster(target: &TargetRef) -> bool {
    target.module.as_str() == KEY && target.kind.as_str() == CLUSTER_KIND
}

pub struct IrrigationTargets {
    trees: Arc<dyn TreeReader>,
    clusters: Arc<dyn TreeClusterReader>,
}

impl IrrigationTargets {
    pub fn new(trees: Arc<dyn TreeReader>, clusters: Arc<dyn TreeClusterReader>) -> Self {
        Self { trees, clusters }
    }

    async fn trees_of(&self, targets: &[TargetRef]) -> Result<Vec<Tree>, RepositoryError> {
        let ids: Vec<Id<Tree>> = targets
            .iter()
            .filter(|t| is_tree(t))
            .map(|t| Id::new(t.id))
            .collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.trees.by_ids(&ids).await
    }

    async fn clusters_of(
        &self,
        ids: Vec<Id<TreeCluster>>,
    ) -> Result<Vec<TreeCluster>, RepositoryError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.clusters.by_ids(&ids).await
    }

    fn cluster_ids(targets: &[TargetRef]) -> Vec<Id<TreeCluster>> {
        targets
            .iter()
            .filter(|t| is_cluster(t))
            .map(|t| Id::new(t.id))
            .collect()
    }
}

#[async_trait::async_trait]
impl TargetCatalog for IrrigationTargets {
    fn kinds(&self) -> &'static [TargetKindSpec] {
        &KINDS
    }

    async fn admit(
        &self,
        target: &TargetRef,
        purpose: TargetPurpose,
    ) -> Result<Id<Organization>, TargetError> {
        match target.kind.as_str() {
            TREE_KIND => {
                let tree = self
                    .trees
                    .by_id(Id::new(target.id))
                    .await
                    .map_err(not_found)?;
                if purpose == TargetPurpose::Sensor && tree.planting_year_in_future(Utc::now()) {
                    return Err(TargetError::Refused {
                        code: "tree.not_yet_planted",
                        message: "a tree that is not planted yet cannot carry a sensor".into(),
                    });
                }
                Ok(tree.organization_id())
            }
            CLUSTER_KIND => Ok(self
                .clusters
                .by_id(Id::new(target.id))
                .await
                .map_err(not_found)?
                .organization_id()),
            _ => Err(TargetError::NotFound),
        }
    }

    async fn locate(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, Coordinate>, RepositoryError> {
        let mut out = HashMap::new();
        for tree in self.trees_of(targets).await? {
            out.insert(tree_target(tree.id), tree.coordinate);
        }
        for cluster in self.clusters_of(Self::cluster_ids(targets)).await? {
            if let Some(coordinate) = cluster.coordinates() {
                out.insert(cluster_target(cluster.id), coordinate);
            }
        }
        Ok(out)
    }

    /// A tree is labelled with its cluster's name; a tree without a cluster
    /// has no label, so the sensor list shows no group line for it.
    async fn label(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, String>, RepositoryError> {
        let trees = self.trees_of(targets).await?;
        let mut cluster_ids = Self::cluster_ids(targets);
        cluster_ids.extend(trees.iter().filter_map(Tree::cluster_id));
        cluster_ids.sort();
        cluster_ids.dedup();
        let clusters: HashMap<Id<TreeCluster>, String> = self
            .clusters_of(cluster_ids)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name.as_str().to_owned()))
            .collect();

        let mut out = HashMap::new();
        for tree in &trees {
            if let Some(name) = tree.cluster_id().and_then(|id| clusters.get(&id)) {
                out.insert(tree_target(tree.id), name.clone());
            }
        }
        for target in targets.iter().filter(|t| is_cluster(t)) {
            if let Some(name) = clusters.get(&Id::new(target.id)) {
                out.insert(target.clone(), name.clone());
            }
        }
        Ok(out)
    }
}

fn not_found(e: RepositoryError) -> TargetError {
    match e {
        RepositoryError::NotFound => TargetError::NotFound,
        e => TargetError::Repository(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn tree_target_uses_the_irrigation_keys() {
        let id = Uuid::now_v7();
        let target = tree_target(Id::new(id));
        assert_eq!(target.module.as_str(), "irrigation");
        assert_eq!(target.kind.as_str(), "tree");
        assert_eq!(target.id, id);
        assert!(is_tree(&target));
    }

    #[test]
    fn cluster_target_uses_the_irrigation_keys() {
        let id = Uuid::now_v7();
        let target = cluster_target(Id::new(id));
        assert_eq!(target.module.as_str(), "irrigation");
        assert_eq!(target.kind.as_str(), "cluster");
        assert_eq!(target.id, id);
        assert!(!is_tree(&target));
    }

    #[test]
    fn only_trees_take_sensors_and_only_clusters_comments() {
        let tree = KINDS.iter().find(|k| k.kind == TREE_KIND).unwrap();
        let cluster = KINDS.iter().find(|k| k.kind == CLUSTER_KIND).unwrap();
        assert!(tree.allows(TargetPurpose::Sensor) && !tree.allows(TargetPurpose::Comment));
        assert!(cluster.allows(TargetPurpose::Comment) && !cluster.allows(TargetPurpose::Sensor));
    }
}
