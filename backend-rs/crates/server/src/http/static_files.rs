//! Serves the embedded frontend single-page app.
//!
//! SPA assets are baked into the release binary via `rust-embed` (only with the
//! `embed-frontend` feature). Requests that match no API route fall through to
//! [`spa_fallback`]: a real asset is returned as-is, everything else returns
//! `index.html` so the client-side router takes over. Unmatched `/api/...`
//! paths are excluded so they keep returning a proper 404, not the SPA shell.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

const INDEX_HTML: &str = "index.html";

/// Returns the embedded asset at `path`, or `index.html` as the SPA fallback.
fn serve_embedded<A: RustEmbed>(path: &str) -> Response {
    let key = path.trim_start_matches('/');
    if let Some(file) = A::get(key) {
        return asset_response(key, file);
    }
    match A::get(INDEX_HTML) {
        Some(index) => asset_response(INDEX_HTML, index),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn asset_response(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    let mime = file.metadata.mimetype();
    let cache = cache_control_for(path);
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, cache)
        .body(Body::from(file.data.into_owned()))
        .expect("asset response headers are always valid")
}

/// Hash-named Vite assets are immutable; the shell and service worker revalidate.
fn cache_control_for(path: &str) -> &'static str {
    if path == INDEX_HTML || path.ends_with("sw.js") || path.ends_with(".webmanifest") {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

/// `/api`, `/swagger-ui`, `/api-docs` must never resolve to the SPA shell.
fn is_reserved_api_path(path: &str) -> bool {
    path == "/api"
        || path.starts_with("/api/")
        || path.starts_with("/swagger-ui")
        || path.starts_with("/api-docs")
}

/// SPA fallback gate: reserved API paths 404, everything else serves the SPA.
fn spa_or_404<A: RustEmbed>(path: &str) -> Response {
    if is_reserved_api_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    serve_embedded::<A>(path)
}

#[cfg(feature = "embed-frontend")]
#[derive(RustEmbed)]
#[folder = "../../../frontend/app/dist"]
struct FrontendAssets;

/// axum fallback handler — serves the embedded SPA for non-API routes.
#[cfg(feature = "embed-frontend")]
pub async fn spa_fallback(uri: axum::http::Uri) -> Response {
    spa_or_404::<FrontendAssets>(uri.path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[derive(RustEmbed)]
    #[folder = "test_assets"]
    struct TestAssets;

    #[test]
    fn cache_control_immutable_for_hashed_assets() {
        assert_eq!(
            cache_control_for("assets/app.abc123.js"),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn cache_control_no_cache_for_shell_and_sw() {
        assert_eq!(cache_control_for("index.html"), "no-cache");
        assert_eq!(cache_control_for("sw.js"), "no-cache");
        assert_eq!(cache_control_for("manifest.webmanifest"), "no-cache");
    }

    #[test]
    fn reserved_api_paths_are_detected() {
        assert!(is_reserved_api_path("/api/v1/tree"));
        assert!(is_reserved_api_path("/api"));
        assert!(is_reserved_api_path("/swagger-ui"));
        assert!(!is_reserved_api_path("/dashboard/trees"));
        assert!(!is_reserved_api_path("/"));
    }

    #[tokio::test]
    async fn serves_hashed_asset_with_immutable_cache() {
        let resp = spa_or_404::<TestAssets>("/assets/app.abc123.js");
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        assert!(body.starts_with(b"console.log"));
    }

    #[tokio::test]
    async fn serves_wasm_with_correct_mime() {
        let resp = spa_or_404::<TestAssets>("/domain.wasm");
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "application/wasm");
    }

    #[tokio::test]
    async fn unknown_path_serves_index_html() {
        let resp = spa_or_404::<TestAssets>("/dashboard/trees/5");
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(ct.starts_with("text/html"));
        assert_eq!(resp.headers()[header::CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn reserved_api_path_returns_404_not_spa() {
        let resp = spa_or_404::<TestAssets>("/api/v1/unknown");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
