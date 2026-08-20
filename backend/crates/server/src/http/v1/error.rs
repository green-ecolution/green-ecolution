use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::service::{AuthError, ServiceError};
use domain::{RepositoryError, routing::RoutingError};

/// The body every failing endpoint returns. Clients parse the response as
/// JSON regardless of status, so error paths must not fall back to plain text.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    /// Stable discriminator for causes a client has to tell apart, e.g. to
    /// pick its own localized wording. Absent where the status and message
    /// already say enough — clients must treat it as optional and never
    /// depend on `error`, which is prose and may be reworded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
}

pub(crate) fn error_response(
    status: StatusCode,
    message: impl Into<String>,
) -> axum::response::Response {
    body_response(status, message, None)
}

/// `error_response` plus the stable `code` discriminator.
pub(crate) fn coded_error_response(
    status: StatusCode,
    message: impl Into<String>,
    code: &'static str,
) -> axum::response::Response {
    body_response(status, message, Some(code))
}

fn body_response(
    status: StatusCode,
    message: impl Into<String>,
    code: Option<&'static str>,
) -> axum::response::Response {
    (
        status,
        axum::Json(ErrorBody {
            error: message.into(),
            code,
        }),
    )
        .into_response()
}

/// Repository error details carry raw driver output (constraint and table
/// names, connection errors). They are logged server-side; clients only ever
/// see the generic per-variant message.
fn repository_error_response(e: &RepositoryError) -> (StatusCode, &'static str) {
    match e {
        RepositoryError::NotFound => (StatusCode::NOT_FOUND, "resource not found"),
        RepositoryError::AlreadyExists(_) => (StatusCode::CONFLICT, "resource already exists"),
        RepositoryError::ForeignKeyViolation(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "referenced resource does not exist",
        ),
        RepositoryError::ConstraintViolation(_) => (
            StatusCode::BAD_REQUEST,
            "request violates a data constraint",
        ),
        RepositoryError::DataIntegrity(_) | RepositoryError::Internal(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AuthError::MissingToken | AuthError::InvalidToken(_) | AuthError::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }
            AuthError::Forbidden => StatusCode::FORBIDDEN,
            AuthError::IdpUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
        if status.is_server_error() {
            tracing::error!(error = %self, kind = "auth", "request failed");
        }
        let body = match &self {
            AuthError::IdpUnavailable(_) => "identity provider unavailable".to_string(),
            other => other.to_string(),
        };
        error_response(status, body)
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServiceError::Repository(e) => {
                let (status, message) = repository_error_response(&e);
                if status.is_server_error() {
                    tracing::error!(error = %e, kind = "repository", "request failed");
                } else {
                    tracing::warn!(error = %e, kind = "repository", "request rejected");
                }
                error_response(status, message)
            }
            ServiceError::InvalidInput(msg) => error_response(StatusCode::BAD_REQUEST, msg),
            ServiceError::Auth(e) => e.into_response(),
            e @ (ServiceError::TreeAlreadyHasSensor
            | ServiceError::SensorAlreadyAssigned
            | ServiceError::AlreadyActivated
            | ServiceError::NotActivated
            | ServiceError::Organization(_)
            | ServiceError::Role(_)
            | ServiceError::OrganizationNotEmpty
            | ServiceError::CannotChangeOwnAccess
            | ServiceError::SensorBoundToTree
            | ServiceError::TreeInCluster) => error_response(StatusCode::CONFLICT, e.to_string()),
            // Not a conflict with stored state: the request combines two
            // entities that may not be linked, which is an input problem.
            ServiceError::OrganizationMismatch(kind) => coded_error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                kind.to_string(),
                kind.code(),
            ),
            e @ (ServiceError::MissingOrganization | ServiceError::ContactPersonNotAMember) => {
                error_response(StatusCode::UNPROCESSABLE_ENTITY, e.to_string())
            }
            ServiceError::Routing(e) => {
                let (status, message) = match &e {
                    RoutingError::Unavailable(_) => (
                        StatusCode::BAD_GATEWAY,
                        "routing engine unavailable".to_string(),
                    ),
                    RoutingError::InvalidProblem(_) => {
                        (StatusCode::UNPROCESSABLE_ENTITY, e.to_string())
                    }
                    RoutingError::Failed(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "route optimization failed".to_string(),
                    ),
                };
                tracing::error!(error = %e, kind = "routing", "request failed");
                error_response(status, message)
            }
            e @ ServiceError::FeatureDisabled { .. } => {
                error_response(StatusCode::SERVICE_UNAVAILABLE, e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::OrganizationMismatch;
    use axum::response::IntoResponse;

    async fn body_of(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn repository_conflict_body_hides_database_details() {
        let detail = "duplicate key value violates unique constraint \"trees_pkey\"";
        let err = ServiceError::Repository(RepositoryError::AlreadyExists(detail.into()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_of(response).await;
        assert!(
            !body.contains("trees_pkey"),
            "constraint name must not leak to the client, got: {body}"
        );
    }

    #[tokio::test]
    async fn repository_fk_violation_body_hides_database_details() {
        let detail = "insert or update on table \"trees\" violates foreign key constraint \"trees_tree_cluster_id_fkey\"";
        let err = ServiceError::Repository(RepositoryError::ForeignKeyViolation(detail.into()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_of(response).await;
        assert!(
            !body.contains("fkey") && !body.contains("\"trees\""),
            "table/constraint names must not leak to the client, got: {body}"
        );
    }

    #[tokio::test]
    async fn repository_internal_body_hides_driver_details() {
        let detail = "error returned from database: connection refused (os error 111)";
        let err = ServiceError::Repository(RepositoryError::Internal(detail.into()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = body_of(response).await;
        assert!(
            !body.contains("connection refused"),
            "driver detail must not leak to the client, got: {body}"
        );
    }

    #[tokio::test]
    async fn organization_mismatch_is_unprocessable_and_carries_its_code() {
        let err = ServiceError::OrganizationMismatch(OrganizationMismatch::TreesVsCluster);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_of(response).await;
        assert!(
            body.contains("organization_mismatch.trees_vs_cluster"),
            "clients pick their wording by code, got: {body}"
        );
        assert!(
            body.contains("cluster"),
            "message must name the violated relation, got: {body}"
        );
    }

    #[test]
    fn every_mismatch_kind_has_a_distinct_code() {
        let kinds = [
            OrganizationMismatch::TreesVsCluster,
            OrganizationMismatch::ClusterVsTree,
            OrganizationMismatch::SensorVsTree,
            OrganizationMismatch::ClustersVsPlan,
            OrganizationMismatch::RoleVsUser,
        ];
        let codes: std::collections::HashSet<&str> = kinds.iter().map(|k| k.code()).collect();
        assert_eq!(codes.len(), kinds.len(), "codes must be unique: {codes:?}");
        assert!(
            kinds
                .iter()
                .all(|k| k.code().starts_with("organization_mismatch.")),
            "codes are namespaced so a client can group them"
        );
    }

    #[tokio::test]
    async fn errors_without_a_code_omit_the_field() {
        let err = ServiceError::TreeInCluster;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = body_of(response).await;
        assert!(
            !body.contains("code"),
            "an absent code must not surface as null, got: {body}"
        );
    }

    #[tokio::test]
    async fn repository_not_found_keeps_status() {
        let err = ServiceError::Repository(RepositoryError::NotFound);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn idp_unavailable_body_hides_transport_details() {
        let err = ServiceError::Auth(AuthError::IdpUnavailable(
            "reqwest::Error { kind: Connect, url: \"http://keycloak:8080\" }".into(),
        ));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = body_of(response).await;
        assert!(
            !body.contains("keycloak:8080"),
            "internal auth URL must not leak to the client, got: {body}"
        );
    }

    #[tokio::test]
    async fn error_body_is_json_so_clients_can_parse_every_response() {
        let err = ServiceError::Repository(RepositoryError::NotFound);
        let response = err.into_response();
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
        let body: serde_json::Value = serde_json::from_str(&body_of(response).await).unwrap();
        assert_eq!(body["error"], "resource not found");
    }

    #[tokio::test]
    async fn auth_error_body_is_json() {
        let response = AuthError::Forbidden.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body: serde_json::Value = serde_json::from_str(&body_of(response).await).unwrap();
        assert!(body["error"].is_string());
    }
}
