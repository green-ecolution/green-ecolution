#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DataExportError {
    #[error("{field} points to a host this instance does not allow")]
    UrlNotAllowed { field: &'static str },
    #[error("the {kind} authentication needs a secret and none is stored yet")]
    SecretMissing { kind: &'static str },
}
