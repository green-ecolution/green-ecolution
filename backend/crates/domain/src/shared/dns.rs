/// A host compared against an allowlist must be a plain DNS name: a URL parser
/// reads characters like `\`, `?` or `@` as delimiters, so a host that merely
/// ends in an allowed suffix could still resolve to a different machine.
pub fn is_dns_name(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            (1..=63).contains(&label.len())
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_dns_names() {
        for host in ["localhost", "frost.localhost", "a-b.example.org", "x1.y2"] {
            assert!(is_dns_name(host), "{host} was refused");
        }
    }

    #[test]
    fn refuses_what_a_url_parser_would_read_differently() {
        for host in [
            "",
            "evil?x.example.org",
            "evil#x.example.org",
            "user@evil.example.org",
            "evil:8080.example.org",
            "frost.localhost.",
            ".frost.localhost",
            "-frost.localhost",
            "frost-.localhost",
            "fro st.localhost",
            "Frost.localhost",
        ] {
            assert!(!is_dns_name(host), "{host} was accepted");
        }
    }

    #[test]
    fn enforces_label_and_total_length() {
        assert!(is_dns_name(&format!("{}.org", "a".repeat(63))));
        assert!(!is_dns_name(&format!("{}.org", "a".repeat(64))));
        let long = format!("{}org", format!("{}.", "a".repeat(60)).repeat(5));
        assert!(long.len() > 253);
        assert!(!is_dns_name(&long));
    }
}
