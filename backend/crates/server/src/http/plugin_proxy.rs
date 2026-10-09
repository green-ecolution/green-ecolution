//! The building blocks of the plugin reverse proxy that need no network:
//! host parsing, the session cookie and header filtering. The request
//! forwarding itself follows in the same module.

use std::{sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use domain::plugin::PluginSlug;

use crate::{
    http::AppState,
    service::{
        AuthError, Feature, ServiceError,
        plugin_proxy_service::{PluginProxyService, ProxyGrant},
    },
};

pub const SESSION_PATH: &str = "/__ge/session";

/// RFC 3986 unreserved characters stay as they are, everything else is encoded.
const HEADER_VALUE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

#[derive(Debug, Clone)]
pub struct PluginHosts {
    suffix: String,
    scheme: String,
    port: Option<u16>,
}

impl PluginHosts {
    pub fn from_public_url(url: &url::Url) -> Option<Self> {
        let host = url.host_str()?.to_ascii_lowercase();
        if url.path() != "/" || url.query().is_some() {
            return None;
        }
        Some(Self {
            suffix: host,
            scheme: url.scheme().to_string(),
            port: url.port(),
        })
    }

    pub fn slug_of(&self, host_header: &str) -> Option<PluginSlug> {
        let host = host_header.to_ascii_lowercase();
        let host = match host.rsplit_once(':') {
            Some((name, port)) if port.chars().all(|c| c.is_ascii_digit()) => name.to_string(),
            _ => host,
        };
        let label = host.strip_suffix(&format!(".{}", self.suffix))?;
        if label.contains('.') {
            return None;
        }
        PluginSlug::new(label).ok()
    }

    pub fn authority_of(&self, slug: &PluginSlug) -> String {
        match self.port {
            Some(port) => format!("{}.{}:{port}", slug.as_str(), self.suffix),
            None => format!("{}.{}", slug.as_str(), self.suffix),
        }
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    pub fn session_url(&self, slug: &PluginSlug, ticket: &str) -> String {
        format!(
            "{}://{}{SESSION_PATH}?ticket={ticket}",
            self.scheme,
            self.authority_of(slug)
        )
    }

    fn is_https(&self) -> bool {
        self.scheme == "https"
    }

    /// `__Host-` requires `Secure`, which browsers do not reliably accept on
    /// plain-http development hosts, so the prefix comes with https only.
    pub fn cookie_name(&self) -> &'static str {
        if self.is_https() {
            "__Host-ge_plugin_session"
        } else {
            "ge_plugin_session"
        }
    }

    pub fn session_cookie(&self, token: &str, max_age_secs: u64) -> HeaderValue {
        let mut cookie = format!(
            "{}={token}; Path=/; Max-Age={max_age_secs}; HttpOnly; SameSite=Lax",
            self.cookie_name()
        );
        if self.is_https() {
            cookie.push_str("; Secure");
        }
        HeaderValue::from_str(&cookie).expect("session cookie is plain ascii")
    }
}

fn cookie_pairs(headers: &HeaderMap) -> impl Iterator<Item = (&str, &str)> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
}

pub fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    cookie_pairs(headers)
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

fn connection_tokens(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(|t| t.trim().to_ascii_lowercase())
        .collect()
}

fn is_hop_by_hop(name: &HeaderName, connection: &[String]) -> bool {
    HOP_BY_HOP.contains(&name.as_str()) || connection.iter().any(|t| t == name.as_str())
}

pub fn upstream_headers(
    incoming: &HeaderMap,
    cookie_name: &str,
    grant: &ProxyGrant,
    forwarded_host: &str,
    forwarded_proto: &str,
) -> HeaderMap {
    let connection = connection_tokens(incoming);
    let mut out = HeaderMap::new();
    // A stale length must not frame a body the client declared as chunked.
    let drop_content_length = incoming.contains_key(header::TRANSFER_ENCODING);
    for (name, value) in incoming {
        let n = name.as_str();
        if is_hop_by_hop(name, &connection)
            || n.contains('_')
            || n.starts_with("x-ge-")
            || (n == "content-length" && drop_content_length)
            || matches!(
                n,
                "host"
                    | "authorization"
                    | "cookie"
                    | "forwarded"
                    | "x-forwarded-host"
                    | "x-forwarded-proto"
                    | "x-forwarded-port"
                    | "x-forwarded-prefix"
                    | "x-forwarded-server"
            )
        {
            continue;
        }
        out.append(name.clone(), value.clone());
    }

    let remaining: Vec<String> = cookie_pairs(incoming)
        .filter(|(k, _)| *k != cookie_name)
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    if !remaining.is_empty()
        && let Ok(value) = HeaderValue::from_str(&remaining.join("; "))
    {
        out.insert(header::COOKIE, value);
    }

    let identity = [
        ("x-ge-user-id", grant.user_id.to_string()),
        (
            "x-ge-user-name",
            utf8_percent_encode(&grant.user_display_name, HEADER_VALUE_SET).to_string(),
        ),
        ("x-forwarded-host", forwarded_host.to_string()),
        ("x-forwarded-proto", forwarded_proto.to_string()),
    ];
    for (name, value) in identity {
        if let Ok(value) = HeaderValue::from_str(&value) {
            out.insert(HeaderName::from_static(name), value);
        }
    }
    out
}

pub fn downstream_headers(upstream: &HeaderMap) -> HeaderMap {
    let connection = connection_tokens(upstream);
    upstream
        .iter()
        .filter(|(name, _)| !is_hop_by_hop(name, &connection))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

/// Only Green Ecolution may frame a proxied view, unless the plugin states
/// its own `frame-ancestors`, which then stands.
pub fn apply_frame_ancestors(headers: &mut HeaderMap, sources: &str) {
    let plugin_sets_it = headers
        .get_all(header::CONTENT_SECURITY_POLICY)
        .iter()
        .map(|v| String::from_utf8_lossy(v.as_bytes()))
        .any(|policy| {
            policy.split(';').any(|directive| {
                directive
                    .split_whitespace()
                    .next()
                    .is_some_and(|name| name.eq_ignore_ascii_case("frame-ancestors"))
            })
        });
    if plugin_sets_it {
        return;
    }
    // A separate header: browsers enforce every CSP header, so the plugin's own policy stays untouched.
    if let Ok(value) = HeaderValue::from_str(&format!("frame-ancestors {sources}")) {
        headers.append(header::CONTENT_SECURITY_POLICY, value);
    }
}

pub struct PluginProxy {
    hosts: PluginHosts,
    client: reqwest::Client,
    frame_ancestors: String,
    session_max_age_secs: u64,
}

impl PluginProxy {
    pub fn new(hosts: PluginHosts, frame_ancestors: String, session_ttl_minutes: u32) -> Self {
        Self {
            hosts,
            // The upstream's redirects belong to the browser, not to us.
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .build()
                .expect("plugin proxy client must build"),
            frame_ancestors,
            session_max_age_secs: u64::from(session_ttl_minutes) * 60,
        }
    }

    pub fn hosts(&self) -> &PluginHosts {
        &self.hosts
    }

    async fn handle(
        &self,
        service: &PluginProxyService,
        slug: &PluginSlug,
        request: Request,
    ) -> Result<Response, ServiceError> {
        if request.uri().path() == SESSION_PATH {
            return self.open_session(service, slug, request.uri()).await;
        }
        let token = read_cookie(request.headers(), self.hosts.cookie_name()).map(str::to_owned);
        let grant = service.authorize(slug, token.as_deref()).await?;
        self.forward(slug, &grant, request).await
    }

    async fn open_session(
        &self,
        service: &PluginProxyService,
        slug: &PluginSlug,
        uri: &Uri,
    ) -> Result<Response, ServiceError> {
        let ticket = uri
            .query()
            .and_then(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .find(|(key, _)| key == "ticket")
                    .map(|(_, value)| value.into_owned())
            })
            .ok_or(AuthError::PluginViewTicketInvalid)?;
        let token = service.open_session(slug, &ticket).await?;
        Ok((
            StatusCode::SEE_OTHER,
            [
                (header::LOCATION, HeaderValue::from_static("/")),
                (
                    header::SET_COOKIE,
                    self.hosts.session_cookie(&token, self.session_max_age_secs),
                ),
            ],
        )
            .into_response())
    }

    async fn forward(
        &self,
        slug: &PluginSlug,
        grant: &ProxyGrant,
        request: Request,
    ) -> Result<Response, ServiceError> {
        let (parts, body) = request.into_parts();
        let path_and_query = parts.uri.path_and_query().map_or("/", |pq| pq.as_str());
        let url = format!(
            "http://{}:{}{path_and_query}",
            grant.endpoint.host(),
            grant.endpoint.port()
        );
        let headers = upstream_headers(
            &parts.headers,
            self.hosts.cookie_name(),
            grant,
            &self.hosts.authority_of(slug),
            self.hosts.scheme(),
        );
        // A GET without a body must not turn into a chunked request upstream.
        let has_body = parts.headers.contains_key(header::CONTENT_LENGTH)
            || parts.headers.contains_key(header::TRANSFER_ENCODING);

        let mut outgoing = self.client.request(parts.method, url).headers(headers);
        if has_body {
            outgoing = outgoing.body(reqwest::Body::wrap_stream(body.into_data_stream()));
        }
        let upstream = outgoing.send().await.map_err(|e| {
            tracing::warn!(error = %e, plugin.slug = %slug.as_str(), "plugin upstream unreachable");
            ServiceError::PluginUpstreamUnreachable
        })?;

        // `bytes_stream` consumes the response, so status and headers go first.
        let status = upstream.status();
        let headers = downstream_headers(upstream.headers());
        let mut response = Response::new(Body::from_stream(upstream.bytes_stream()));
        *response.status_mut() = status;
        *response.headers_mut() = headers;
        Ok(response)
    }
}

/// Requests to `<slug>.<suffix>` never reach the API router; everything else
/// passes through untouched.
pub async fn dispatch(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let (Some(proxy), Some(service)) = (
        state.plugin_proxy.clone(),
        state.plugin_proxy_service.clone(),
    ) else {
        return next.run(request).await;
    };
    // HTTP/2 carries the host in `:authority` and sends no `Host` header.
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .or_else(|| request.uri().authority().map(|a| a.as_str()));
    let Some(slug) = host.and_then(|h| proxy.hosts.slug_of(h)) else {
        return next.run(request).await;
    };

    let mut response = if state.feature_flags.plugins_enabled {
        proxy
            .handle(&service, &slug, request)
            .await
            .unwrap_or_else(IntoResponse::into_response)
    } else {
        ServiceError::FeatureDisabled {
            feature: Feature::Plugins,
        }
        .into_response()
    };
    apply_frame_ancestors(response.headers_mut(), &proxy.frame_ancestors);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderName, HeaderValue, header};
    use domain::plugin::ServiceEndpoint;

    fn hosts(url: &str) -> PluginHosts {
        PluginHosts::from_public_url(&url.parse().unwrap()).unwrap()
    }

    fn grant(name: &str) -> ProxyGrant {
        ProxyGrant {
            endpoint: ServiceEndpoint::new("kataster.plugins.svc", 8080).unwrap(),
            user_id: uuid::Uuid::nil(),
            user_display_name: name.into(),
        }
    }

    #[test]
    fn public_url_needs_a_host_and_no_path() {
        assert!(
            PluginHosts::from_public_url(&"https://plugins.example.org/ui".parse().unwrap())
                .is_none()
        );
        assert!(
            PluginHosts::from_public_url(&"https://plugins.example.org/".parse().unwrap())
                .is_some()
        );
    }

    #[test]
    fn slug_comes_from_the_label_left_of_the_suffix() {
        let h = hosts("https://plugins.example.org");
        assert_eq!(
            h.slug_of("kataster.plugins.example.org").unwrap().as_str(),
            "kataster"
        );
        assert_eq!(
            h.slug_of("Kataster.Plugins.Example.org:443")
                .unwrap()
                .as_str(),
            "kataster"
        );
    }

    #[test]
    fn foreign_and_nested_hosts_are_not_plugin_hosts() {
        let h = hosts("https://plugins.example.org");
        assert!(h.slug_of("app.example.org").is_none());
        assert!(h.slug_of("plugins.example.org").is_none());
        assert!(h.slug_of("a.b.plugins.example.org").is_none());
        assert!(h.slug_of("evilplugins.example.org").is_none());
        assert!(h.slug_of("Bad_Slug.plugins.example.org").is_none());
    }

    #[test]
    fn session_url_keeps_scheme_and_non_default_port() {
        let slug = PluginSlug::new("demo").unwrap();
        assert_eq!(
            hosts("http://plugins.localhost:3000").session_url(&slug, "gev_ab"),
            "http://demo.plugins.localhost:3000/__ge/session?ticket=gev_ab"
        );
        assert_eq!(
            hosts("https://plugins.example.org").session_url(&slug, "gev_ab"),
            "https://demo.plugins.example.org/__ge/session?ticket=gev_ab"
        );
    }

    #[test]
    fn cookie_is_host_prefixed_and_secure_only_under_https() {
        let secure = hosts("https://plugins.example.org");
        assert_eq!(secure.cookie_name(), "__Host-ge_plugin_session");
        assert_eq!(
            secure.session_cookie("gps_x", 60).to_str().unwrap(),
            "__Host-ge_plugin_session=gps_x; Path=/; Max-Age=60; HttpOnly; SameSite=Lax; Secure"
        );
        let plain = hosts("http://plugins.localhost:3000");
        assert_eq!(
            plain.session_cookie("gps_x", 60).to_str().unwrap(),
            "ge_plugin_session=gps_x; Path=/; Max-Age=60; HttpOnly; SameSite=Lax"
        );
    }

    #[test]
    fn read_cookie_finds_the_value_across_headers() {
        let mut h = HeaderMap::new();
        h.append(header::COOKIE, HeaderValue::from_static("theme=dark"));
        h.append(
            header::COOKIE,
            HeaderValue::from_static("a=1; ge_plugin_session=gps_y"),
        );
        assert_eq!(read_cookie(&h, "ge_plugin_session"), Some("gps_y"));
        assert_eq!(read_cookie(&h, "missing"), None);
    }

    #[test]
    fn upstream_headers_strip_credentials_hop_by_hop_and_forged_identity() {
        let mut incoming = HeaderMap::new();
        incoming.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer ey"));
        incoming.insert("x-ge-user-id", HeaderValue::from_static("forged"));
        incoming.insert("X-GE-Anything", HeaderValue::from_static("forged"));
        incoming.insert(
            header::CONNECTION,
            HeaderValue::from_static("keep-alive, x-secret"),
        );
        incoming.insert("x-secret", HeaderValue::from_static("1"));
        incoming.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
        incoming.insert(
            header::HOST,
            HeaderValue::from_static("demo.plugins.example.org"),
        );
        incoming.insert(
            header::COOKIE,
            HeaderValue::from_static("ge_plugin_session=gps_y; theme=dark"),
        );
        incoming.insert(header::ACCEPT, HeaderValue::from_static("text/html"));
        incoming.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.7"));

        let out = upstream_headers(
            &incoming,
            "ge_plugin_session",
            &grant("Toni"),
            "demo.plugins.example.org",
            "https",
        );

        assert!(out.get(header::AUTHORIZATION).is_none());
        assert!(out.get("x-ge-anything").is_none());
        assert!(out.get("x-secret").is_none());
        assert!(out.get(header::CONNECTION).is_none());
        assert!(out.get(header::TRANSFER_ENCODING).is_none());
        assert!(out.get(header::HOST).is_none());
        assert_eq!(out.get(header::COOKIE).unwrap(), "theme=dark");
        assert_eq!(out.get(header::ACCEPT).unwrap(), "text/html");
        assert_eq!(out.get("x-forwarded-for").unwrap(), "203.0.113.7");
        assert_eq!(
            out.get("x-ge-user-id").unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            out.get("x-forwarded-host").unwrap(),
            "demo.plugins.example.org"
        );
        assert_eq!(out.get("x-forwarded-proto").unwrap(), "https");
    }

    #[test]
    fn display_name_is_percent_encoded() {
        let out = upstream_headers(
            &HeaderMap::new(),
            "c",
            &grant("Jörg Müller-Lüdenscheidt"),
            "h",
            "https",
        );
        assert_eq!(
            out.get("x-ge-user-name").unwrap(),
            "J%C3%B6rg%20M%C3%BCller-L%C3%BCdenscheidt"
        );
    }

    #[test]
    fn a_session_cookie_alone_leaves_no_cookie_header() {
        let mut incoming = HeaderMap::new();
        incoming.insert(
            header::COOKIE,
            HeaderValue::from_static("ge_plugin_session=gps_y"),
        );
        let out = upstream_headers(&incoming, "ge_plugin_session", &grant("T"), "h", "http");
        assert!(out.get(header::COOKIE).is_none());
    }

    #[test]
    fn downstream_headers_drop_hop_by_hop_only() {
        let mut upstream = HeaderMap::new();
        upstream.insert(header::CONNECTION, HeaderValue::from_static("close"));
        upstream.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
        upstream.insert(header::SET_COOKIE, HeaderValue::from_static("plugin=1"));
        upstream.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
        let out = downstream_headers(&upstream);
        assert!(out.get(header::CONNECTION).is_none());
        assert!(out.get(header::TRANSFER_ENCODING).is_none());
        assert_eq!(out.get(header::SET_COOKIE).unwrap(), "plugin=1");
        assert_eq!(out.get(header::CONTENT_TYPE).unwrap(), "text/html");
    }

    #[test]
    fn frame_ancestors_is_added_or_appended_but_never_overrides() {
        let mut none = HeaderMap::new();
        apply_frame_ancestors(&mut none, "https://app.example.org");
        assert_eq!(
            none.get(header::CONTENT_SECURITY_POLICY).unwrap(),
            "frame-ancestors https://app.example.org"
        );

        let mut other = HeaderMap::new();
        other.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        );
        apply_frame_ancestors(&mut other, "https://app.example.org");
        let values: Vec<_> = other
            .get_all(header::CONTENT_SECURITY_POLICY)
            .iter()
            .collect();
        assert_eq!(
            values,
            [
                "default-src 'self'",
                "frame-ancestors https://app.example.org"
            ]
        );

        let mut own = HeaderMap::new();
        own.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("Frame-Ancestors 'none'"),
        );
        apply_frame_ancestors(&mut own, "https://app.example.org");
        assert_eq!(
            own.get(header::CONTENT_SECURITY_POLICY).unwrap(),
            "Frame-Ancestors 'none'"
        );
    }

    #[test]
    fn underscore_header_names_and_extra_forwarding_headers_are_dropped() {
        let mut incoming = HeaderMap::new();
        for (k, v) in [
            ("x_ge_user_id", "forged"),
            ("X_GE_User_Name", "forged"),
            ("x_forwarded_host", "evil"),
            ("forwarded", "for=evil"),
            ("x-forwarded-port", "1"),
            ("x-forwarded-prefix", "/p"),
            ("x-forwarded-server", "evil"),
        ] {
            incoming.insert(
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_static(v),
            );
        }
        let out = upstream_headers(&incoming, "c", &grant("T"), "h", "https");
        for k in [
            "x_ge_user_id",
            "x_ge_user_name",
            "x_forwarded_host",
            "forwarded",
            "x-forwarded-port",
            "x-forwarded-prefix",
            "x-forwarded-server",
        ] {
            assert!(out.get(k).is_none(), "{k} leaked");
        }
        assert_eq!(
            out.get("x-ge-user-id").unwrap(),
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(out.get("x-forwarded-host").unwrap(), "h");
    }

    #[test]
    fn content_length_is_dropped_only_next_to_transfer_encoding() {
        let mut both = HeaderMap::new();
        both.insert(header::CONTENT_LENGTH, HeaderValue::from_static("5"));
        both.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
        assert!(
            upstream_headers(&both, "c", &grant("T"), "h", "http")
                .get(header::CONTENT_LENGTH)
                .is_none()
        );

        let mut only = HeaderMap::new();
        only.insert(header::CONTENT_LENGTH, HeaderValue::from_static("5"));
        assert_eq!(
            upstream_headers(&only, "c", &grant("T"), "h", "http")
                .get(header::CONTENT_LENGTH)
                .unwrap(),
            "5"
        );
    }

    #[test]
    fn frame_ancestors_in_a_later_csp_header_is_respected() {
        let mut h = HeaderMap::new();
        h.append(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'"),
        );
        h.append(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("frame-ancestors 'none'"),
        );
        apply_frame_ancestors(&mut h, "https://app.example.org");
        assert_eq!(h.get_all(header::CONTENT_SECURITY_POLICY).iter().count(), 2);
    }

    #[test]
    fn frame_ancestors_inside_a_url_does_not_count() {
        let mut h = HeaderMap::new();
        h.insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; report-uri https://r.example/frame-ancestors",
            ),
        );
        apply_frame_ancestors(&mut h, "https://app.example.org");
        assert_eq!(h.get_all(header::CONTENT_SECURITY_POLICY).iter().count(), 2);
    }
}
