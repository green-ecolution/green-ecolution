use std::net::IpAddr;

use url::{Host, Url};

use super::error::DataExportError;
use crate::{RepositoryError, shared::dns::is_dns_name, shared::error::ValidationError};

pub const MAX_URL_LEN: usize = 2048;

/// A URL the server will call on behalf of an organization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportUrl(Url);

impl ExportUrl {
    pub fn new(field: &'static str, raw: &str) -> Result<Self, ValidationError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(ValidationError::EmptyString { field });
        }
        let too_long = |got: usize| ValidationError::TooLong {
            field,
            max: MAX_URL_LEN,
            got,
        };
        if raw.len() > MAX_URL_LEN {
            return Err(too_long(raw.len()));
        }
        let invalid = |reason: &str| ValidationError::InvalidFormat {
            field,
            reason: reason.to_string(),
        };
        let url = Url::parse(raw).map_err(|_| invalid("not a valid URL"))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(invalid("scheme must be http or https"));
        }
        if url.host().is_none() {
            return Err(invalid("must name a host"));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(invalid("must not carry credentials"));
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(invalid("must not carry a query or fragment"));
        }
        // Percent-encoding can push a short input past the limit.
        if url.as_str().len() > MAX_URL_LEN {
            return Err(too_long(url.as_str().len()));
        }
        Ok(Self(url))
    }

    /// A SensorThings service root, spelled without a trailing `/` so that
    /// `…/v1.1/` and `…/v1.1` are one target. Not for token URLs, where the
    /// slash may be significant and redirects are not followed.
    pub fn service_root(field: &'static str, raw: &str) -> Result<Self, ValidationError> {
        let mut url = Self::new(field, raw)?.0;
        let path = url.path().trim_end_matches('/').to_string();
        url.set_path(&path);
        Ok(Self(url))
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Scheme, host and port (the scheme's default when none is written).
    pub fn same_origin(&self, other: &ExportUrl) -> bool {
        self.0.origin() == other.0.origin()
    }

    #[doc(hidden)]
    pub fn reconstitute(raw: &str) -> Result<Self, RepositoryError> {
        Url::parse(raw)
            .map(Self)
            .map_err(|e| RepositoryError::DataIntegrity(format!("stored export URL: {e}")))
    }
}

/// Which hosts an organization may point the export at. Without it anyone
/// allowed to edit an organization's settings could make the server call
/// internal services or the cloud metadata endpoint.
///
/// An entry is a host, a `.suffix` for every host below it, or an IP literal,
/// optionally followed by `:port` or `:*`. Without a port only the scheme's
/// default port is allowed, so a listed host cannot be used to reach its other
/// services.
#[derive(Debug, Clone)]
pub struct ExportUrlPolicy {
    allowed: Vec<AllowedHost>,
    allow_insecure_http: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AllowedHost {
    host: String,
    port: PortRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PortRule {
    SchemeDefault,
    Exactly(u16),
    Any,
}

impl PortRule {
    fn allows(self, url: &Url) -> bool {
        match self {
            // `Url::port` is `None` when the URL names no port or the scheme's default.
            Self::SchemeDefault => url.port().is_none(),
            Self::Exactly(port) => url.port_or_known_default() == Some(port),
            Self::Any => true,
        }
    }
}

impl ExportUrlPolicy {
    pub fn new(allowed_hosts: impl IntoIterator<Item = String>, allow_insecure_http: bool) -> Self {
        Self {
            allowed: allowed_hosts
                .into_iter()
                .filter_map(|h| parse_entry(&h))
                .collect(),
            allow_insecure_http,
        }
    }

    /// Entries that no URL can match: a scheme, path, userinfo or malformed port
    /// has no place in a host list.
    pub fn unusable_entries(raw: &[String]) -> Vec<String> {
        raw.iter()
            .filter(|entry| !entry.trim().is_empty() && parse_entry(entry).is_none())
            .cloned()
            .collect()
    }

    pub fn check(&self, field: &'static str, url: &ExportUrl) -> Result<(), DataExportError> {
        let url = url.as_url();
        let scheme_ok = url.scheme() == "https" || self.allow_insecure_http;
        let allowed = self
            .allowed
            .iter()
            .any(|entry| entry.matches_host(url) && entry.port.allows(url));
        if scheme_ok && allowed {
            Ok(())
        } else {
            Err(DataExportError::UrlNotAllowed { field })
        }
    }
}

impl AllowedHost {
    fn matches_host(&self, url: &Url) -> bool {
        match url.host() {
            Some(Host::Domain(domain)) => {
                let domain = domain.to_ascii_lowercase();
                is_dns_name(&domain)
                    && (domain == self.host
                        || domain
                            .strip_suffix(self.host.as_str())
                            .is_some_and(|rest| rest.ends_with('.')))
            }
            Some(Host::Ipv4(ip)) => self.host == ip.to_string(),
            Some(Host::Ipv6(ip)) => self.host == ip.to_string(),
            None => false,
        }
    }
}

/// IP literals are compared by their canonical text, so `[::1]` and the
/// expanded form both equal what `Url` reports for the host.
fn parse_entry(raw: &str) -> Option<AllowedHost> {
    let entry = raw.trim().trim_start_matches('.').to_ascii_lowercase();
    if entry.is_empty() || entry.contains(['/', '@']) {
        return None;
    }
    let canonical = |host: &str| {
        host.parse::<IpAddr>()
            .map_or_else(|_| host.to_string(), |ip| ip.to_string())
    };
    if let Ok(ip) = entry.parse::<IpAddr>() {
        return Some(AllowedHost {
            host: ip.to_string(),
            port: PortRule::SchemeDefault,
        });
    }
    let (host, port) = match entry.strip_prefix('[') {
        Some(bracketed) => {
            let (ip, rest) = bracketed.split_once(']')?;
            ip.parse::<std::net::Ipv6Addr>().ok()?;
            match rest {
                "" => (ip, None),
                rest => (ip, Some(rest.strip_prefix(':')?)),
            }
        }
        None => match entry.rsplit_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (entry.as_str(), None),
        },
    };
    if host.is_empty() || host.contains(':') && host.parse::<IpAddr>().is_err() {
        return None;
    }
    let port = match port {
        None => PortRule::SchemeDefault,
        Some("*") => PortRule::Any,
        Some(p) => PortRule::Exactly(p.parse().ok()?),
    };
    Some(AllowedHost {
        host: canonical(host),
        port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::error::ValidationError;

    const F: &str = "data_export.base_url";

    fn url(raw: &str) -> ExportUrl {
        ExportUrl::new(F, raw).unwrap()
    }

    fn policy(hosts: &[&str], insecure: bool) -> ExportUrlPolicy {
        ExportUrlPolicy::new(hosts.iter().map(|h| h.to_string()), insecure)
    }

    #[test]
    fn accepts_a_sensorthings_root() {
        let u = url(" https://frost.example.org/FROST-Server/v1.1 ");
        assert_eq!(u.as_str(), "https://frost.example.org/FROST-Server/v1.1");
    }

    #[test]
    fn rejects_malformed_and_unsupported_urls() {
        for raw in [
            "frost.example.org",
            "ftp://frost.example.org",
            "https://user:pw@frost.example.org",
            "https://user@frost.example.org",
            "https://frost.example.org/v1.1?x=1",
            "https://frost.example.org/v1.1#top",
            "file:///etc/passwd",
        ] {
            assert!(
                matches!(
                    ExportUrl::new(F, raw),
                    Err(ValidationError::InvalidFormat { field: F, .. })
                ),
                "{raw} was accepted"
            );
        }
    }

    #[test]
    fn rejects_empty_and_overlong_urls() {
        assert_eq!(
            ExportUrl::new(F, "  "),
            Err(ValidationError::EmptyString { field: F })
        );
        let long = format!("https://frost.example.org/{}", "a".repeat(2048));
        assert!(matches!(
            ExportUrl::new(F, &long),
            Err(ValidationError::TooLong {
                field: F,
                max: 2048,
                ..
            })
        ));
    }

    #[test]
    fn the_limit_applies_to_the_encoded_url() {
        let raw = format!("https://frost.example.org/{}", "ä".repeat(1000));
        assert!(raw.len() <= MAX_URL_LEN);
        assert!(matches!(
            ExportUrl::new(F, &raw),
            Err(ValidationError::TooLong {
                field: F,
                max: 2048,
                ..
            })
        ));
    }

    #[test]
    fn a_service_root_drops_its_trailing_slash_except_for_the_root() {
        let root = |raw| ExportUrl::service_root(F, raw).unwrap();
        assert_eq!(
            root("https://frost.example.org/FROST-Server/v1.1/").as_str(),
            "https://frost.example.org/FROST-Server/v1.1"
        );
        assert_eq!(
            root("https://frost.example.org/v1.1//").as_str(),
            "https://frost.example.org/v1.1"
        );
        assert_eq!(
            root("https://frost.example.org/").as_str(),
            "https://frost.example.org/"
        );
        assert_eq!(
            root("https://frost.example.org").as_str(),
            "https://frost.example.org/"
        );
        assert!(ExportUrl::service_root(F, "https://user@frost.example.org/v1.1").is_err());
    }

    #[test]
    fn other_urls_keep_their_trailing_slash() {
        assert_eq!(
            url("https://auth.example.org/o/token/").as_str(),
            "https://auth.example.org/o/token/"
        );
    }

    #[test]
    fn allows_an_exact_host_and_hosts_below_a_suffix() {
        let p = policy(&["example.org", ".platform.test:8443"], false);
        assert!(p.check(F, &url("https://example.org/v1.1")).is_ok());
        assert!(p.check(F, &url("https://frost.example.org/v1.1")).is_ok());
        assert!(
            p.check(F, &url("https://a.b.platform.test:8443/v1.1"))
                .is_ok()
        );
    }

    #[test]
    fn an_entry_without_port_allows_only_the_scheme_default() {
        let p = policy(&["example.org"], true);
        assert!(p.check(F, &url("https://example.org:443/v1.1")).is_ok());
        assert!(p.check(F, &url("http://example.org/v1.1")).is_ok());
        for raw in [
            "https://example.org:8443/v1.1",
            "https://example.org:80/v1.1",
            "http://frost.example.org:5432/",
        ] {
            assert!(p.check(F, &url(raw)).is_err(), "{raw} was allowed");
        }
    }

    #[test]
    fn a_port_entry_allows_that_port_and_a_star_any() {
        let p = policy(&["example.org:8443", "dev.test:*"], false);
        assert!(p.check(F, &url("https://example.org:8443/v1.1")).is_ok());
        assert!(p.check(F, &url("https://example.org/v1.1")).is_err());
        assert!(p.check(F, &url("https://dev.test:1234/v1.1")).is_ok());
        assert!(p.check(F, &url("https://dev.test/v1.1")).is_ok());
    }

    #[test]
    fn refuses_hosts_off_the_list() {
        let p = policy(&["example.org"], false);
        for raw in [
            "https://evilexample.org/v1.1",
            "https://example.org.evil.test/v1.1",
            "https://db/v1.1",
            "https://169.254.169.254/latest",
            "https://[::1]/v1.1",
        ] {
            assert_eq!(
                p.check(F, &url(raw)),
                Err(DataExportError::UrlNotAllowed { field: F }),
                "{raw} was allowed"
            );
        }
    }

    #[test]
    fn plain_http_needs_the_instance_switch() {
        let u = url("http://frost.localhost:8180/FROST-Server/v1.1");
        assert!(policy(&["localhost:8180"], false).check(F, &u).is_err());
        assert!(policy(&["localhost:8180"], true).check(F, &u).is_ok());
    }

    #[test]
    fn ip_literals_pass_only_when_listed_verbatim() {
        let u = url("http://127.0.0.1:9000/v1.1");
        assert!(policy(&["127.0.0.1:9000"], true).check(F, &u).is_ok());
        assert!(policy(&["localhost:9000"], true).check(F, &u).is_err());
        // The parser normalizes shorthand IPv4, so it cannot slip past the list.
        assert!(
            policy(&["localhost"], true)
                .check(F, &url("http://127.1/v1.1"))
                .is_err()
        );
    }

    #[test]
    fn ip_entries_match_in_any_spelling() {
        let plain = url("http://[::1]/v1.1");
        for entry in ["::1", "[::1]", "0:0:0:0:0:0:0:1", "[0:0:0:0:0:0:0:1]"] {
            assert!(policy(&[entry], true).check(F, &plain).is_ok(), "{entry}");
        }
        let ported = url("http://[::1]:8080/v1.1");
        for entry in ["[::1]:8080", "[0:0:0:0:0:0:0:1]:8080", "[::1]:*"] {
            assert!(policy(&[entry], true).check(F, &ported).is_ok(), "{entry}");
        }
        assert!(policy(&["::1"], true).check(F, &ported).is_err());
    }

    #[test]
    fn names_entries_that_can_never_match() {
        let raw = [
            "https://x.org",
            "x.org:443",
            "x.org:*",
            "x.org",
            "x.org/v1",
            "a@x.org",
            "x.org:http",
            "x.org:70000",
            "[::1]",
            "[::1]:8080",
            "::1",
            "[x]:1",
        ]
        .map(String::from);
        assert_eq!(
            ExportUrlPolicy::unusable_entries(&raw),
            [
                "https://x.org",
                "x.org/v1",
                "a@x.org",
                "x.org:http",
                "x.org:70000",
                "[x]:1"
            ]
        );
    }

    #[test]
    fn an_empty_policy_allows_nothing() {
        assert!(
            policy(&[], true)
                .check(F, &url("https://example.org/v1.1"))
                .is_err()
        );
    }
}
