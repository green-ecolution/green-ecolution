use domain::plugin::ServiceEndpoint;

/// Which in-cluster services a `proxied` plugin may point at. Without it an
/// administrator could aim the proxy at the cloud metadata endpoint or any
/// internal service and have Green Ecolution fetch it on their behalf.
#[derive(Debug, Clone)]
pub struct ProxyPolicy {
    service_suffixes: Vec<String>,
    ports: Vec<u16>,
}

impl ProxyPolicy {
    pub fn new(
        service_suffixes: impl IntoIterator<Item = String>,
        ports: impl IntoIterator<Item = u16>,
    ) -> Self {
        Self {
            service_suffixes: service_suffixes
                .into_iter()
                .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            ports: ports.into_iter().collect(),
        }
    }

    pub fn allows(&self, endpoint: &ServiceEndpoint) -> bool {
        let host = endpoint.host().to_ascii_lowercase();
        self.ports.contains(&endpoint.port())
            && self.service_suffixes.iter().any(|suffix| {
                host == *suffix
                    || host
                        .strip_suffix(suffix.as_str())
                        .is_some_and(|rest| rest.ends_with('.'))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ProxyPolicy {
        ProxyPolicy::new(
            [
                ".plugins.svc.cluster.local".to_string(),
                "demo-plugin".to_string(),
            ],
            [80, 8080],
        )
    }

    fn ep(host: &str, port: u16) -> ServiceEndpoint {
        ServiceEndpoint::new(host, port).unwrap()
    }

    #[test]
    fn allows_a_service_below_a_suffix() {
        assert!(policy().allows(&ep("kataster.plugins.svc.cluster.local", 8080)));
    }

    #[test]
    fn allows_an_exact_entry() {
        assert!(policy().allows(&ep("demo-plugin", 80)));
    }

    #[test]
    fn a_leading_dot_in_the_entry_is_optional() {
        assert!(policy().allows(&ep("plugins.svc.cluster.local", 80)));
    }

    #[test]
    fn refuses_a_host_that_merely_ends_with_the_entry() {
        assert!(!policy().allows(&ep("evil-demo-plugin", 80)));
    }

    #[test]
    fn refuses_a_port_off_the_list() {
        assert!(!policy().allows(&ep("demo-plugin", 22)));
    }

    #[test]
    fn refuses_an_unlisted_host() {
        assert!(!policy().allows(&ep("169.254.169.254", 80)));
    }

    #[test]
    fn host_comparison_ignores_case() {
        assert!(policy().allows(&ep("Demo-Plugin", 80)));
    }

    #[test]
    fn an_empty_policy_allows_nothing() {
        assert!(
            !ProxyPolicy::new(Vec::<String>::new(), Vec::<u16>::new())
                .allows(&ep("demo-plugin", 80))
        );
    }
}
