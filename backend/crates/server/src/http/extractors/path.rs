use axum::{
    extract::{FromRequestParts, rejection::PathRejection},
    http::request::Parts,
    response::{IntoResponse, Response},
};

use crate::http::v1::error::coded_error_response;

/// Drop-in replacement for [`axum::extract::Path`] that renders extraction
/// failures as the API's JSON error body. Axum's own rejection is `text/plain`,
/// which breaks clients that parse every response as JSON.
pub struct Path<T>(pub T);

impl<T, S> FromRequestParts<S> for Path<T>
where
    T: serde::de::DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = PathParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(value) =
            axum::extract::Path::<T>::from_request_parts(parts, state).await?;
        Ok(Self(value))
    }
}

pub struct PathParamRejection(PathRejection);

impl From<PathRejection> for PathParamRejection {
    fn from(value: PathRejection) -> Self {
        Self(value)
    }
}

impl IntoResponse for PathParamRejection {
    fn into_response(self) -> Response {
        tracing::warn!(error = %self.0, kind = "path_param", "request rejected");
        coded_error_response(
            self.0.status(),
            self.0.body_text(),
            "request.malformed_path_parameter",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
        routing::get,
    };
    use tower::ServiceExt;

    async fn handler(Path(id): Path<uuid::Uuid>) -> String {
        id.to_string()
    }

    async fn call(uri: &str) -> axum::response::Response {
        Router::new()
            .route("/{id}", get(handler))
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_malformed_path_parameter_is_rendered_as_json() {
        let response = call("/not-a-uuid").await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json"),
            "clients parse every response as JSON"
        );

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], "request.malformed_path_parameter");
        assert!(body["error"].is_string());
    }

    #[tokio::test]
    async fn a_well_formed_path_parameter_still_reaches_the_handler() {
        let id = uuid::Uuid::now_v7();
        let response = call(&format!("/{id}")).await;

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), id.to_string());
    }
}
