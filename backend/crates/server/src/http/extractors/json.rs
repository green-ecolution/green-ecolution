use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};

use crate::http::v1::error::error_response;

/// Drop-in replacement for [`axum::Json`] that renders extraction failures as
/// the API's JSON error body. Axum's own rejections are `text/plain`, which
/// breaks clients that parse every response as JSON.
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonBodyRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req, state).await?;
        Ok(Self(value))
    }
}

impl<T: serde::Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub struct JsonBodyRejection(JsonRejection);

impl From<JsonRejection> for JsonBodyRejection {
    fn from(value: JsonRejection) -> Self {
        Self(value)
    }
}

impl IntoResponse for JsonBodyRejection {
    fn into_response(self) -> Response {
        tracing::warn!(error = %self.0, kind = "request_body", "request rejected");
        error_response(self.0.status(), self.0.body_text())
    }
}
