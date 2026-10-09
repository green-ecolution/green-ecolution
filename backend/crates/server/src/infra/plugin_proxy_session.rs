use rand::Rng;
use sha2::{Digest, Sha256};

use crate::service::plugin_proxy_service::ProxySessionFactory;

const PREFIX: &str = "gps_";

/// The production `ProxySessionFactory`: 32 bytes from the OS CSPRNG, stored
/// as a SHA-256 digest of the whole token.
pub struct RandomProxySessionFactory;

impl ProxySessionFactory for RandomProxySessionFactory {
    fn generate(&self) -> (String, String) {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let token = format!("{PREFIX}{}", to_hex(&bytes));
        let hash = hash_token(&token);
        (token, hash)
    }

    fn hash(&self, token: &str) -> Option<String> {
        token.starts_with(PREFIX).then(|| hash_token(token))
    }
}

fn hash_token(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_carries_prefix_and_256_bits() {
        let (token, _) = RandomProxySessionFactory.generate();
        assert!(token.starts_with("gps_"));
        assert_eq!(token.len(), 4 + 64);
    }

    #[test]
    fn hash_matches_the_generated_token() {
        let (token, hash) = RandomProxySessionFactory.generate();
        assert_eq!(RandomProxySessionFactory.hash(&token), Some(hash));
    }

    #[test]
    fn a_view_ticket_is_not_a_session_token() {
        assert_eq!(RandomProxySessionFactory.hash("gev_abc"), None);
    }
}
