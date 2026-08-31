use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::extract::QueryRejection;

use crate::http::v1::error::coded_error_response;

/// Drop-in replacement for [`axum_extra::extract::Query`] that renders extraction
/// failures as the API's JSON error body. Axum's own rejection is `text/plain`,
/// which breaks clients that parse every response as JSON.
///
/// Wraps the `axum_extra` variant, not `axum`s: it also accepts a repeated key
/// as a list, which the list endpoints rely on for multi-value filters.
pub struct Query<T>(pub T);

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = QueryParamRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let axum_extra::extract::Query(value) =
            axum_extra::extract::Query::<T>::from_request_parts(parts, state).await?;
        Ok(Self(value))
    }
}

pub struct QueryParamRejection(QueryRejection);

impl From<QueryRejection> for QueryParamRejection {
    fn from(value: QueryRejection) -> Self {
        Self(value)
    }
}

impl IntoResponse for QueryParamRejection {
    fn into_response(self) -> Response {
        tracing::warn!(error = %self.0, kind = "query_param", "request rejected");
        coded_error_response(
            self.0.status(),
            self.0.body_text(),
            "request.malformed_query_parameter",
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

    #[derive(serde::Deserialize)]
    struct Params {
        page: u32,
    }

    async fn handler(Query(params): Query<Params>) -> String {
        params.page.to_string()
    }

    async fn call(uri: &str) -> axum::response::Response {
        Router::new()
            .route("/", get(handler))
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_malformed_query_parameter_is_rendered_as_json() {
        let response = call("/?page=not-a-number").await;

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
        assert_eq!(body["code"], "request.malformed_query_parameter");
    }

    #[tokio::test]
    async fn a_well_formed_query_parameter_still_reaches_the_handler() {
        let response = call("/?page=3").await;

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), "3");
    }
}
