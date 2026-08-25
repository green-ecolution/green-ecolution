pub mod authorization;
pub mod cluster_service;
pub mod evaluation_service;
pub mod event_bus;
pub mod handlers;
pub mod organization_service;
pub mod region_service;
pub mod role_service;
pub mod sensor_service;
pub mod start_point_service;
pub mod tree_service;
pub mod user_service;
pub mod vehicle_service;
pub mod watering_execution_service;
pub mod watering_plan_service;

use domain::{RepositoryError, routing::RoutingError, shared::error::ValidationError};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("tree already has a different sensor")]
    TreeAlreadyHasSensor,
    #[error("sensor is already assigned to another tree")]
    SensorAlreadyAssigned,
    #[error("sensor is already activated")]
    AlreadyActivated,
    #[error("sensor is not activated")]
    NotActivated,
    #[error("{feature} feature is disabled")]
    FeatureDisabled { feature: &'static str },
    #[error(transparent)]
    Routing(#[from] RoutingError),
    #[error(transparent)]
    Organization(#[from] domain::organization::OrganizationError),
    #[error(transparent)]
    Role(#[from] domain::role::RoleError),
    #[error("organization still has sub-organizations or users")]
    OrganizationNotEmpty,
    #[error(transparent)]
    OrganizationMismatch(#[from] OrganizationMismatch),
    #[error("contact person is not a member of this organization")]
    ContactPersonNotAMember,
    #[error("tree is part of a cluster")]
    TreeInCluster,
    #[error("sensor is bound to a tree and must be transferred with it")]
    SensorBoundToTree,
    #[error("no organization given and the acting user has none")]
    MissingOrganization,
    #[error("a user cannot change their own roles or organization")]
    CannotChangeOwnAccess,
    #[error("this change would remove your own right to administer roles and members")]
    CannotRevokeOwnAdministration,
}

/// Which cross-aggregate organization rule a request violated.
///
/// Two entities may only be linked while they belong to the same
/// organization; a request that would cross that boundary is rejected here
/// rather than silently pulling one side along. The variants exist so clients
/// can tell the cases apart — the `code` is a stable wire contract, the
/// `Display` text is a fallback for clients without their own wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OrganizationMismatch {
    #[error("the selected trees belong to a different organization than the cluster")]
    TreesVsCluster,
    #[error("the selected cluster belongs to a different organization than the tree")]
    ClusterVsTree,
    #[error("the sensor belongs to a different organization than the tree")]
    SensorVsTree,
    #[error("the selected clusters are outside the watering plan's organization")]
    ClustersVsPlan,
    #[error("the role belongs to a different organization than the user")]
    RoleVsUser,
}

impl OrganizationMismatch {
    pub fn code(self) -> &'static str {
        match self {
            Self::TreesVsCluster => "organization_mismatch.trees_vs_cluster",
            Self::ClusterVsTree => "organization_mismatch.cluster_vs_tree",
            Self::SensorVsTree => "organization_mismatch.sensor_vs_tree",
            Self::ClustersVsPlan => "organization_mismatch.clusters_vs_plan",
            Self::RoleVsUser => "organization_mismatch.role_vs_user",
        }
    }
}

impl From<ValidationError> for ServiceError {
    fn from(err: ValidationError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing or malformed authorization header")]
    MissingToken,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("forbidden: missing required permission")]
    Forbidden,
    #[error("identity provider unavailable: {0}")]
    IdpUnavailable(String),
}
