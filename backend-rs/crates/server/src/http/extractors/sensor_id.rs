use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use domain::sensor::SensorId;

use crate::service::ServiceError;

/// Axum path extractor that parses a `{sensor_id}` URL segment into a
/// validated [`SensorId`]. Rejects malformed paths with `400 Bad Request`.
pub struct SensorIdPath(pub SensorId);

impl<S> FromRequestParts<S> for SensorIdPath
where
    S: Send + Sync,
{
    type Rejection = ServiceError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(raw) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|e| ServiceError::InvalidInput(format!("invalid sensor id path: {e}")))?;
        Ok(Self(SensorId::new(raw)?))
    }
}

impl std::fmt::Display for SensorIdPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
