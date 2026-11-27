use rand::RngExt;
use secrecy::SecretString;

use crate::service::user_service::InitialPasswordFactory;

// Without 0/O, 1/l/I: the password is read off a screen and typed once.
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const DIGIT: &[u8] = b"23456789";
const GROUPS: usize = 4;
const GROUP_LEN: usize = 5;

/// The production `InitialPasswordFactory`: 20 characters from the OS CSPRNG
/// in four groups, with at least one upper-case letter, one lower-case letter
/// and one digit so common realm policies accept it.
pub struct RandomInitialPassword;

impl InitialPasswordFactory for RandomInitialPassword {
    fn generate(&self) -> SecretString {
        let alphabet: Vec<u8> = [UPPER, LOWER, DIGIT].concat();
        let mut rng = rand::rng();
        loop {
            let chars: Vec<u8> = (0..GROUPS * GROUP_LEN)
                .map(|_| alphabet[rng.random_range(0..alphabet.len())])
                .collect();
            let has = |set: &[u8]| chars.iter().any(|c| set.contains(c));
            if has(UPPER) && has(LOWER) && has(DIGIT) {
                let text = chars
                    .chunks(GROUP_LEN)
                    .map(|group| String::from_utf8_lossy(group).into_owned())
                    .collect::<Vec<_>>()
                    .join("-");
                return SecretString::from(text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    fn password() -> String {
        RandomInitialPassword.generate().expose_secret().to_string()
    }

    #[test]
    fn has_four_groups_of_five() {
        for _ in 0..200 {
            let groups: Vec<String> = password().split('-').map(str::to_string).collect();
            assert_eq!(groups.len(), GROUPS);
            assert!(groups.iter().all(|g| g.len() == GROUP_LEN));
        }
    }

    #[test]
    fn uses_only_the_alphabet_and_has_every_kind() {
        let alphabet: Vec<u8> = [UPPER, LOWER, DIGIT].concat();
        for _ in 0..200 {
            let text = password();
            let chars: Vec<u8> = text.bytes().filter(|c| *c != b'-').collect();
            assert!(chars.iter().all(|c| alphabet.contains(c)), "{text}");
            for set in [UPPER, LOWER, DIGIT] {
                assert!(chars.iter().any(|c| set.contains(c)), "{text}");
            }
        }
    }

    #[test]
    fn two_calls_differ() {
        assert_ne!(password(), password());
    }
}
