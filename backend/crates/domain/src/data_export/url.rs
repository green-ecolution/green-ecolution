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
        if raw.len() > MAX_URL_LEN {
            return Err(ValidationError::TooLong {
                field,
                max: MAX_URL_LEN,
                got: raw.len(),
            });
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
#[derive(Debug, Clone)]
pub struct ExportUrlPolicy {
    allowed_hosts: Vec<String>,
    allow_insecure_http: bool,
}

impl ExportUrlPolicy {
    pub fn new(allowed_hosts: impl IntoIterator<Item = String>, allow_insecure_http: bool) -> Self {
        Self {
            allowed_hosts: allowed_hosts
                .into_iter()
                .map(|h| h.trim().trim_start_matches('.').to_ascii_lowercase())
                .filter(|h| !h.is_empty())
                .collect(),
            allow_insecure_http,
        }
    }

    pub fn check(&self, field: &'static str, url: &ExportUrl) -> Result<(), DataExportError> {
        let url = url.as_url();
        let scheme_ok = url.scheme() == "https" || self.allow_insecure_http;
        let host_ok = match url.host() {
            Some(Host::Domain(domain)) => {
                let domain = domain.to_ascii_lowercase();
                is_dns_name(&domain)
                    && self.allowed_hosts.iter().any(|entry| {
                        domain == *entry
                            || domain
                                .strip_suffix(entry.as_str())
                                .is_some_and(|rest| rest.ends_with('.'))
                    })
            }
            Some(Host::Ipv4(ip)) => self.allowed_hosts.contains(&ip.to_string()),
            Some(Host::Ipv6(ip)) => self.allowed_hosts.contains(&ip.to_string()),
            None => false,
        };
        if scheme_ok && host_ok {
            Ok(())
        } else {
            Err(DataExportError::UrlNotAllowed { field })
        }
    }
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
    fn allows_an_exact_host_and_hosts_below_a_suffix() {
        let p = policy(&["example.org", ".platform.test"], false);
        assert!(p.check(F, &url("https://example.org/v1.1")).is_ok());
        assert!(p.check(F, &url("https://frost.example.org/v1.1")).is_ok());
        assert!(
            p.check(F, &url("https://a.b.platform.test:8443/v1.1"))
                .is_ok()
        );
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
        assert!(policy(&["localhost"], false).check(F, &u).is_err());
        assert!(policy(&["localhost"], true).check(F, &u).is_ok());
    }

    #[test]
    fn ip_literals_pass_only_when_listed_verbatim() {
        let u = url("http://127.0.0.1:9000/v1.1");
        assert!(policy(&["127.0.0.1"], true).check(F, &u).is_ok());
        assert!(policy(&["localhost"], true).check(F, &u).is_err());
        // The parser normalizes shorthand IPv4, so it cannot slip past the list.
        assert!(
            policy(&["localhost"], true)
                .check(F, &url("http://127.1/v1.1"))
                .is_err()
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
