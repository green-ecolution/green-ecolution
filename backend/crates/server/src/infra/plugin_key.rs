use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use domain::Id;
use domain::plugin::{Plugin, PluginKeyHash};

use crate::service::plugin_service::PluginKeyFactory;

const PREFIX: &str = "gep_";

/// The production `PluginKeyFactory`: 32 bytes from the OS CSPRNG, stored as
/// a SHA-256 digest.
pub struct RandomPluginKeyFactory;

impl PluginKeyFactory for RandomPluginKeyFactory {
    fn generate(&self, plugin: Id<Plugin>) -> (String, PluginKeyHash) {
        generate_key(plugin)
    }
}

/// Returns the plaintext key (shown to the operator exactly once) and the hash
/// to persist.
///
/// SHA-256 rather than a password hash: the secret is 32 random bytes, so
/// there is nothing to guess, and a work factor would cost every ingest
/// request three-digit milliseconds.
pub fn generate_key(plugin: Id<Plugin>) -> (String, PluginKeyHash) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let secret = to_hex(&bytes);
    let plaintext = format!("{PREFIX}{}.{secret}", plugin.value());
    let hash = hash_secret(&secret);
    (plaintext, hash)
}

pub fn hash_secret(secret: &str) -> PluginKeyHash {
    PluginKeyHash::reconstitute(to_hex(&Sha256::digest(secret.as_bytes())))
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

/// Splits `gep_<uuid>.<secret>`. Returns `None` for anything else, including a
/// user's JWT, so the extractor can fall through cleanly.
pub fn parse_key(raw: &str) -> Option<(Uuid, String)> {
    let rest = raw.strip_prefix(PREFIX)?;
    let (id, secret) = rest.split_once('.')?;
    if secret.is_empty() {
        return None;
    }
    Some((Uuid::parse_str(id).ok()?, secret.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::Id;

    #[test]
    fn generated_key_parses_back_to_its_plugin() {
        let id = Id::new_v7();
        let (plaintext, _) = generate_key(id);
        let (parsed_id, _secret) = parse_key(&plaintext).expect("generated key must parse");
        assert_eq!(parsed_id, id.value());
    }

    #[test]
    fn generated_key_carries_the_prefix() {
        let (plaintext, _) = generate_key(Id::new_v7());
        assert!(plaintext.starts_with("gep_"));
    }

    #[test]
    fn hash_matches_the_generated_secret() {
        let id = Id::new_v7();
        let (plaintext, hash) = generate_key(id);
        let (_, secret) = parse_key(&plaintext).unwrap();
        assert_eq!(hash_secret(&secret).as_str(), hash.as_str());
    }

    #[test]
    fn two_keys_differ() {
        let id = Id::new_v7();
        let (a, _) = generate_key(id);
        let (b, _) = generate_key(id);
        assert_ne!(a, b);
    }

    #[test]
    fn parse_rejects_foreign_shapes() {
        assert!(parse_key("Bearer abc").is_none());
        assert!(parse_key("gep_no-dot").is_none());
        assert!(parse_key("gep_not-a-uuid.secret").is_none());
    }
}
