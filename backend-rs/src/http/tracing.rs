use std::time::Duration;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue, Response},
};
use tower_http::request_id::{MakeRequestId, RequestId};
use tracing::Span;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Clone, Copy, Default)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &axum::http::Request<B>) -> Option<RequestId> {
        let id = HeaderValue::from_str(&Uuid::new_v4().to_string()).ok()?;
        Some(RequestId::new(id))
    }
}

pub fn make_span(request: &Request) -> Span {
    let request_id = request
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    tracing::info_span!(
        "http_request",
        "http.request.method" = %request.method(),
        "url.path" = %request.uri().path(),
        "url.query" = request.uri().query(),
        "network.protocol.version" = ?request.version(),
        request_id = %request_id,
        "http.response.status_code" = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
    )
}

pub fn on_response(response: &Response<Body>, latency: Duration, span: &Span) {
    span.record("http.response.status_code", response.status().as_u16());
    span.record("latency_ms", latency.as_millis() as u64);
}
