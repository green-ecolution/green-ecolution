//! Small SQL helpers shared across the Postgres repositories.

/// Escapes `\`, `%`, `_` for use inside an ILIKE pattern with `ESCAPE '\'`.
///
/// Backslash must be escaped first so an injected `\%` does not survive as a
/// usable wildcard.
pub(crate) fn like_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str(r"\\"),
            '%' => out.push_str(r"\%"),
            '_' => out.push_str(r"\_"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod like_escape_tests {
    use super::like_escape;

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(like_escape(""), "");
    }

    #[test]
    fn plain_input_unchanged() {
        assert_eq!(like_escape("Eiche"), "Eiche");
        assert_eq!(like_escape("T-001"), "T-001");
    }

    #[test]
    fn escapes_percent() {
        assert_eq!(like_escape("50%"), r"50\%");
    }

    #[test]
    fn escapes_underscore() {
        assert_eq!(like_escape("a_b"), r"a\_b");
    }

    #[test]
    fn escapes_backslash_first() {
        assert_eq!(like_escape(r"a\b"), r"a\\b");
    }

    #[test]
    fn escapes_combined() {
        assert_eq!(like_escape(r"50%\_x"), r"50\%\\\_x");
    }
}
