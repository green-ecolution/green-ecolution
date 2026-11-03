#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataExportError {
    #[error("{field} points to a host this instance does not allow")]
    UrlNotAllowed { field: &'static str },
    #[error("the {kind} authentication needs a secret and none is stored for this destination")]
    SecretMissing { kind: &'static str },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_export::{BASE_URL_FIELD, TOKEN_URL_FIELD};

    #[test]
    fn a_refused_url_names_its_field_because_the_dialog_reads_it() {
        for field in [BASE_URL_FIELD, TOKEN_URL_FIELD] {
            assert!(
                DataExportError::UrlNotAllowed { field }
                    .to_string()
                    .contains(field)
            );
        }
    }
}
