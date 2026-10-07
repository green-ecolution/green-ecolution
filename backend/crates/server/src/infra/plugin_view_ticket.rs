use rand::Rng;
use sha2::{Digest, Sha256};

use crate::service::plugin_view_ticket_service::ViewTicketFactory;

const PREFIX: &str = "gev_";

/// The production `ViewTicketFactory`: 32 bytes from the OS CSPRNG, stored as
/// a SHA-256 digest of the whole ticket.
pub struct RandomViewTicketFactory;

impl ViewTicketFactory for RandomViewTicketFactory {
    fn generate(&self) -> (String, String) {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let ticket = format!("{PREFIX}{}", to_hex(&bytes));
        let hash = hash_ticket(&ticket);
        (ticket, hash)
    }

    fn hash(&self, ticket: &str) -> Option<String> {
        ticket.starts_with(PREFIX).then(|| hash_ticket(ticket))
    }
}

fn hash_ticket(ticket: &str) -> String {
    to_hex(&Sha256::digest(ticket.as_bytes()))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ticket_carries_prefix_and_256_bits() {
        let (ticket, _) = RandomViewTicketFactory.generate();
        assert!(ticket.starts_with("gev_"));
        assert_eq!(ticket.len(), 4 + 64);
    }

    #[test]
    fn hash_matches_the_generated_ticket() {
        let (ticket, hash) = RandomViewTicketFactory.generate();
        assert_eq!(RandomViewTicketFactory.hash(&ticket), Some(hash));
    }

    #[test]
    fn two_tickets_differ() {
        let (a, _) = RandomViewTicketFactory.generate();
        let (b, _) = RandomViewTicketFactory.generate();
        assert_ne!(a, b);
    }

    #[test]
    fn foreign_shapes_have_no_hash() {
        assert_eq!(RandomViewTicketFactory.hash("gep_not-a-ticket"), None);
        assert_eq!(RandomViewTicketFactory.hash(""), None);
    }
}
