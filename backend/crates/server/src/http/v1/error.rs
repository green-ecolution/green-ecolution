use axum::{http::StatusCode, response::IntoResponse};

use crate::service::{AuthError, ServiceError};
use domain::{RepositoryError, routing::RoutingError};

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
        (status, body).into_response()
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
                (status, message).into_response()
            }
            ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ServiceError::Auth(e) => e.into_response(),
            e @ (ServiceError::TreeAlreadyHasSensor
            | ServiceError::SensorAlreadyAssigned
            | ServiceError::AlreadyActivated
            | ServiceError::NotActivated
            | ServiceError::Organization(_)
            | ServiceError::Role(_)
            | ServiceError::OrganizationNotEmpty
            | ServiceError::OrganizationMismatch
            | ServiceError::TreeInCluster) => (StatusCode::CONFLICT, e.to_string()).into_response(),
            e @ (ServiceError::ShareTargetNotDescendant | ServiceError::MissingOrganization) => {
                (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
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
                (status, message).into_response()
            }
            e @ ServiceError::FeatureDisabled { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
