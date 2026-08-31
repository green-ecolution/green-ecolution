use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::service::{AuthError, ServiceError};
use domain::{RepositoryError, routing::RoutingError, shared::error::ValidationIssue};

/// OpenAPI shape of [`ValidationIssue`].
///
/// A mirror rather than a derive on the domain type: the domain crate must
/// build without utoipa, so the schema cannot live next to the struct. Keep the
/// fields in step with `domain::shared::error::ValidationIssue`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(as = ValidationIssue)]
#[schema(example = json!({
    "field": "cluster.name",
    "key": "cluster.name.tooLong",
    "params": { "max": 255, "got": 300 }
}))]
pub struct ValidationIssueSchema {
    /// Namespaced domain field label.
    pub field: String,
    /// Translation key, `{field}.{violated rule}`.
    pub key: String,
    /// Values the translated sentence interpolates.
    pub params: serde_json::Value,
}

/// The body every failing endpoint returns. Clients parse the response as
/// JSON regardless of status, so error paths must not fall back to plain text.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "error": "tree is part of a cluster",
    "code": "conflict.tree_in_cluster"
}))]
pub struct ErrorBody {
    /// Prose for logs and for a client without its own wording. May be
    /// reworded at any time; branch on `code` instead.
    pub error: String,
    /// Stable discriminator every failing response carries, so a client can
    /// pick its own localized wording instead of showing `error`, which is
    /// prose and may be reworded. Still optional on the wire: a client that
    /// does not know a code must fall back on the status.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub code: Option<&'static str>,
    /// Which input field broke which rule, present only when one field is to
    /// blame. Carries the same `key` and `params` the in-browser validator
    /// emits, so one catalog entry serves both paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<ValidationIssueSchema>)]
    pub validation: Option<ValidationIssue>,
}

/// Renders the error body with its stable `code` discriminator.
pub(crate) fn coded_error_response(
    status: StatusCode,
    message: impl Into<String>,
    code: &'static str,
) -> axum::response::Response {
    body_response(status, message, Some(code), None)
}

fn body_response(
    status: StatusCode,
    message: impl Into<String>,
    code: Option<&'static str>,
    validation: Option<ValidationIssue>,
) -> axum::response::Response {
    (
        status,
        axum::Json(ErrorBody {
            error: message.into(),
            code,
            validation,
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

/// Status and client-facing message for an auth failure. Shared by both
/// `IntoResponse` impls so `ServiceError::Auth` cannot drift from `AuthError`.
fn auth_error_response(e: &AuthError) -> (StatusCode, String) {
    let status = match e {
        AuthError::MissingToken | AuthError::InvalidToken(_) | AuthError::TokenExpired => {
            StatusCode::UNAUTHORIZED
        }
        AuthError::Forbidden => StatusCode::FORBIDDEN,
        AuthError::IdpUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
    };
    let message = match e {
        AuthError::IdpUnavailable(_) => "identity provider unavailable".to_string(),
        other => other.to_string(),
    };
    (status, message)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = auth_error_response(&self);
        if status.is_server_error() {
            tracing::error!(error = %self, kind = "auth", "request failed");
        }
        coded_error_response(status, message, self.code())
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code();
        let validation = self.validation_issue();
        let (status, message) = match &self {
            ServiceError::Repository(e) => {
                let (status, message) = repository_error_response(e);
                if status.is_server_error() {
                    tracing::error!(error = %e, kind = "repository", "request failed");
                } else {
                    tracing::warn!(error = %e, kind = "repository", "request rejected");
                }
                (status, message.to_string())
            }
            ServiceError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ServiceError::Malformed { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            ServiceError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ServiceError::Auth(e) => {
                let (status, message) = auth_error_response(e);
                if status.is_server_error() {
                    tracing::error!(error = %e, kind = "auth", "request failed");
                }
                (status, message)
            }
            ServiceError::TreeAlreadyHasSensor
            | ServiceError::SensorAlreadyAssigned
            | ServiceError::AlreadyActivated
            | ServiceError::NotActivated
            | ServiceError::Organization(_)
            | ServiceError::Role(_)
            | ServiceError::OrganizationNotEmpty
            | ServiceError::CannotChangeOwnAccess
            | ServiceError::CannotRevokeOwnAdministration
            | ServiceError::SensorBoundToTree
            | ServiceError::TreeInCluster => (StatusCode::CONFLICT, self.to_string()),
            // Not a conflict with stored state: the request combines two
            // entities that may not be linked, which is an input problem.
            ServiceError::OrganizationMismatch(kind) => {
                (StatusCode::UNPROCESSABLE_ENTITY, kind.to_string())
            }
            ServiceError::MissingOrganization | ServiceError::ContactPersonNotAMember => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            // Status kept at 400 as before; the transition is now only
            // distinguishable by its code, not by a new status.
            ServiceError::WateringPlan(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            ServiceError::Routing(e) => {
                let (status, message) = match e {
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
                (status, message)
            }
            ServiceError::FeatureDisabled { .. } => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string())
            }
        };
        body_response(status, message, Some(code), validation)
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
    async fn a_conflict_names_its_cause_so_the_client_can_word_it() {
        let err = ServiceError::TreeInCluster;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body: serde_json::Value = serde_json::from_str(&body_of(response).await).unwrap();
        assert_eq!(
            body["code"], "conflict.tree_in_cluster",
            "a bare 409 cannot tell twelve causes apart"
        );
    }

    #[tokio::test]
    async fn a_plan_state_transition_is_distinguishable_from_other_bad_input() {
        use domain::watering_plan::{WateringPlanError, WateringPlanStatus};

        let err = ServiceError::WateringPlan(WateringPlanError::InvalidStateTransition {
            from: WateringPlanStatus::Finished,
            to: WateringPlanStatus::Active,
        });
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value = serde_json::from_str(&body_of(response).await).unwrap();
        assert_eq!(body["code"], "watering_plan.invalid_state_transition");
    }

    #[tokio::test]
    async fn each_disabled_feature_has_its_own_code() {
        use crate::service::Feature;

        let routing = ServiceError::FeatureDisabled {
            feature: Feature::Routing,
        }
        .into_response();
        assert_eq!(routing.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: serde_json::Value = serde_json::from_str(&body_of(routing).await).unwrap();
        assert_eq!(body["code"], "feature.routing_disabled");
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

#[cfg(test)]
mod validation_tests {
    use super::*;
    use axum::response::IntoResponse;
    use domain::shared::error::ValidationError;

    async fn body_of(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn a_field_violation_travels_as_key_and_params() {
        let err = ServiceError::from(ValidationError::TooLong {
            field: "cluster.name",
            max: 255,
            got: 300,
        });
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = body_of(response).await;
        assert_eq!(body["code"], "request.validation_failed");
        assert_eq!(body["validation"]["field"], "cluster.name");
        assert_eq!(body["validation"]["key"], "cluster.name.tooLong");
        assert_eq!(body["validation"]["params"]["max"], 255);
        assert_eq!(body["validation"]["params"]["got"], 300);
    }

    #[tokio::test]
    async fn an_out_of_range_violation_carries_its_bounds() {
        let err = ServiceError::from(ValidationError::OutOfRange {
            field: "coordinate.latitude",
            min: -90.0,
            max: 90.0,
            got: 91.0,
        });
        let body = body_of(err.into_response()).await;
        assert_eq!(body["validation"]["key"], "coordinate.latitude.outOfRange");
        assert_eq!(body["validation"]["params"]["got"], 91.0);
    }

    #[tokio::test]
    async fn an_empty_field_violation_carries_no_parameters() {
        let err = ServiceError::from(ValidationError::EmptyString {
            field: "tree.species",
        });
        let body = body_of(err.into_response()).await;
        assert_eq!(body["validation"]["key"], "tree.species.empty");
        assert_eq!(body["validation"]["params"], serde_json::json!({}));
    }

    #[tokio::test]
    async fn a_format_violation_carries_its_reason() {
        let err = ServiceError::from(ValidationError::InvalidFormat {
            field: "user.email",
            reason: "missing @".into(),
        });
        let body = body_of(err.into_response()).await;
        assert_eq!(body["validation"]["key"], "user.email.invalidFormat");
        assert_eq!(body["validation"]["params"]["reason"], "missing @");
    }

    #[tokio::test]
    async fn a_too_short_violation_carries_the_minimum() {
        let err = ServiceError::from(ValidationError::TooShort {
            field: "role.name",
            min: 2,
            got: 1,
        });
        let body = body_of(err.into_response()).await;
        assert_eq!(body["validation"]["key"], "role.name.tooShort");
        assert_eq!(body["validation"]["params"]["min"], 2);
    }

    #[tokio::test]
    async fn an_error_without_a_field_omits_the_validation_block() {
        let body = body_of(ServiceError::TreeInCluster.into_response()).await;
        assert!(
            body.get("validation").is_none(),
            "only field violations carry a validation block, got: {body}"
        );
    }
}
