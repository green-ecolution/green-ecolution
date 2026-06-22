//! Serves the embedded frontend single-page app.
//!
//! SPA assets are baked into the release binary via `rust-embed` (only with the
//! `embed-frontend` feature). Requests that match no API route fall through to
//! [`spa_fallback`]: a real asset is returned as-is, everything else returns
//! `index.html` so the client-side router takes over. Unmatched `/api/...`
//! paths are excluded so they keep returning a proper 404, not the SPA shell.
//!
//! Each asset carries a strong `ETag` (the embedded sha256) so revalidatable
//! responses (`index.html`, the service worker, the manifest) answer a matching
//! `If-None-Match` with `304 Not Modified` instead of resending the body.

use axum::{
    body::{Body, Bytes},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

const INDEX_HTML: &str = "index.html";

fn serve_embedded<A: RustEmbed>(path: &str, if_none_match: Option<&str>) -> Response {
    let key = path.trim_start_matches('/');
    if let Some(file) = A::get(key) {
        return asset_response(key, file, if_none_match);
    }
    match A::get(INDEX_HTML) {
        Some(index) => asset_response(INDEX_HTML, index, if_none_match),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn asset_response(
    path: &str,
    file: rust_embed::EmbeddedFile,
    if_none_match: Option<&str>,
) -> Response {
    let etag = etag_for(&file);
    let cache = cache_control_for(path);

    if if_none_match.is_some_and(|m| m == etag) {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, &etag)
            .header(header::CACHE_CONTROL, cache)
            .body(Body::empty())
            .expect("304 response headers are always valid");
    }

    let content_type = content_type_for(file.metadata.mimetype());
    // In release builds `data` borrows the binary's rodata; serve it zero-copy
    // instead of cloning the whole asset on every request.
    let body = match file.data {
        std::borrow::Cow::Borrowed(bytes) => Body::from(Bytes::from_static(bytes)),
        std::borrow::Cow::Owned(bytes) => Body::from(bytes),
    };
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache)
        .header(header::ETAG, &etag)
        .body(body)
        .expect("asset response headers are always valid")
}

fn etag_for(file: &rust_embed::EmbeddedFile) -> String {
    use std::fmt::Write;
    let hash = file.metadata.sha256_hash();
    let mut etag = String::with_capacity(2 + hash.len() * 2);
    etag.push('"');
    for byte in hash {
        let _ = write!(etag, "{byte:02x}");
    }
    etag.push('"');
    etag
}

/// `mime_guess` omits the charset; text assets must declare UTF-8 explicitly.
fn content_type_for(mime: &str) -> String {
    if mime.starts_with("text/") {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_owned()
    }
}

/// Hash-named Vite assets are immutable; the shell, service worker and manifest
/// revalidate (cheaply, via ETag).
fn cache_control_for(path: &str) -> &'static str {
    if path == INDEX_HTML
        || path.ends_with("sw.js")
        || path.ends_with(".webmanifest")
        || path.ends_with("manifest.json")
    {
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

fn spa_or_404<A: RustEmbed>(path: &str, if_none_match: Option<&str>) -> Response {
    if is_reserved_api_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    serve_embedded::<A>(path, if_none_match)
}

#[cfg(feature = "embed-frontend")]
#[derive(RustEmbed)]
#[folder = "../../../frontend/app/dist"]
struct FrontendAssets;

#[cfg(feature = "embed-frontend")]
pub async fn spa_fallback(uri: axum::http::Uri, headers: axum::http::HeaderMap) -> Response {
    let if_none_match = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok());
    spa_or_404::<FrontendAssets>(uri.path(), if_none_match)
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
    fn cache_control_no_cache_for_shell_sw_and_manifest() {
        assert_eq!(cache_control_for("index.html"), "no-cache");
        assert_eq!(cache_control_for("sw.js"), "no-cache");
        assert_eq!(cache_control_for("manifest.webmanifest"), "no-cache");
        assert_eq!(cache_control_for("manifest.json"), "no-cache");
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
        let resp = spa_or_404::<TestAssets>("/assets/app.abc123.js", None);
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
        let resp = spa_or_404::<TestAssets>("/domain.wasm", None);
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "application/wasm");
    }

    #[tokio::test]
    async fn unknown_path_serves_index_html() {
        let resp = spa_or_404::<TestAssets>("/dashboard/trees/5", None);
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers()[header::CONTENT_TYPE].to_str().unwrap();
        assert!(ct.starts_with("text/html"));
        assert!(ct.contains("charset=utf-8"));
        assert_eq!(resp.headers()[header::CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn reserved_api_path_returns_404_not_spa() {
        let resp = spa_or_404::<TestAssets>("/api/v1/unknown", None);
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn matching_if_none_match_returns_304() {
        let first = spa_or_404::<TestAssets>("/assets/app.abc123.js", None);
        let etag = first.headers()[header::ETAG].to_str().unwrap().to_owned();

        let cached = spa_or_404::<TestAssets>("/assets/app.abc123.js", Some(&etag));
        assert_eq!(cached.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(cached.headers()[header::ETAG], etag);

        let body = to_bytes(cached.into_body(), usize::MAX).await.unwrap();
        assert!(body.is_empty());
    }
}
