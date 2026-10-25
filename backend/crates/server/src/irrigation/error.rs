use domain::{RepositoryError, shared::error::ValidationError};
use irrigation_domain::{cluster::ClusterError, tree::TreeError};

use crate::service::{ModuleErrorKind, ServiceError};

#[derive(Debug, thiserror::Error)]
pub enum IrrigationError {
    #[error(transparent)]
    Core(#[from] ServiceError),
    #[error("tree is part of a cluster")]
    TreeInCluster,
    #[error("tree is not planted yet and cannot carry a sensor")]
    TreeNotYetPlanted,
    #[error("the selected trees belong to a different organization than the cluster")]
    TreesVsCluster,
    #[error("the selected cluster belongs to a different organization than the tree")]
    ClusterVsTree,
}

impl From<IrrigationError> for ServiceError {
    fn from(err: IrrigationError) -> Self {
        use ModuleErrorKind::{Conflict, Unprocessable};
        let (kind, code) = match err {
            IrrigationError::Core(inner) => return inner,
            IrrigationError::TreeInCluster => (Conflict, "conflict.tree_in_cluster"),
            IrrigationError::TreeNotYetPlanted => (Unprocessable, "tree.not_yet_planted"),
            // Not a conflict with stored state: the request links two
            // entities that may not be linked, which is an input problem.
            IrrigationError::TreesVsCluster => {
                (Unprocessable, "organization_mismatch.trees_vs_cluster")
            }
            IrrigationError::ClusterVsTree => {
                (Unprocessable, "organization_mismatch.cluster_vs_tree")
            }
        };
        ServiceError::Module {
            kind,
            code,
            message: err.to_string(),
        }
    }
}

impl From<RepositoryError> for IrrigationError {
    fn from(e: RepositoryError) -> Self {
        Self::Core(ServiceError::Repository(e))
    }
}

impl From<ValidationError> for IrrigationError {
    fn from(e: ValidationError) -> Self {
        Self::Core(ServiceError::Validation(e))
    }
}

/// Only `NotYetPlanted` actually travels this way: the calibration variants
/// are consumed by the reading handler and never reach a service.
impl From<TreeError> for IrrigationError {
    fn from(e: TreeError) -> Self {
        match e {
            TreeError::NotYetPlanted => Self::TreeNotYetPlanted,
            TreeError::Validation(v) => Self::Core(ServiceError::Validation(v)),
            other => Self::Core(ServiceError::InvalidInput(other.to_string())),
        }
    }
}

impl From<ClusterError> for IrrigationError {
    fn from(e: ClusterError) -> Self {
        match e {
            ClusterError::Validation(v) => Self::Core(ServiceError::Validation(v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::*;

    async fn body_of(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn irrigation_errors_keep_their_public_codes() {
        let cases = [
            (IrrigationError::TreeInCluster, "conflict.tree_in_cluster"),
            (IrrigationError::TreeNotYetPlanted, "tree.not_yet_planted"),
            (
                IrrigationError::TreesVsCluster,
                "organization_mismatch.trees_vs_cluster",
            ),
            (
                IrrigationError::ClusterVsTree,
                "organization_mismatch.cluster_vs_tree",
            ),
        ];
        for (err, code) in cases {
            assert_eq!(ServiceError::from(err).code(), code);
        }
    }

    #[test]
    fn a_core_error_passes_through_unchanged() {
        let err = ServiceError::from(IrrigationError::from(RepositoryError::NotFound));
        assert!(matches!(
            err,
            ServiceError::Repository(RepositoryError::NotFound)
        ));
    }

    #[test]
    fn a_tree_that_is_not_planted_yet_maps_to_its_own_variant() {
        assert!(matches!(
            IrrigationError::from(TreeError::NotYetPlanted),
            IrrigationError::TreeNotYetPlanted
        ));
    }

    #[tokio::test]
    async fn a_conflict_names_its_cause_so_the_client_can_word_it() {
        let response = ServiceError::from(IrrigationError::TreeInCluster).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_of(response).await;
        assert_eq!(
            body["code"], "conflict.tree_in_cluster",
            "a bare 409 cannot tell twelve causes apart"
        );
        assert!(body.get("validation").is_none());
    }

    #[tokio::test]
    async fn a_mismatch_is_unprocessable_and_names_the_relation() {
        for err in [
            IrrigationError::TreesVsCluster,
            IrrigationError::ClusterVsTree,
        ] {
            let response = ServiceError::from(err).into_response();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
            let body = body_of(response).await;
            assert!(
                body["code"]
                    .as_str()
                    .unwrap()
                    .starts_with("organization_mismatch."),
                "clients pick their wording by code, got: {body}"
            );
            assert!(
                body["error"].as_str().unwrap().contains("cluster"),
                "message must name the violated relation, got: {body}"
            );
        }
    }

    #[tokio::test]
    async fn a_tree_not_yet_planted_is_unprocessable() {
        let response = ServiceError::from(IrrigationError::TreeNotYetPlanted).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
