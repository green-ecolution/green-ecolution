use domain::{RepositoryError, shared::error::ValidationError};
use server::service::{ModuleErrorKind, ServiceError};

#[derive(Debug, thiserror::Error)]
pub enum WasteError {
    #[error(transparent)]
    Core(#[from] ServiceError),
    #[error("a waste bin with this number already exists in the organization")]
    NumberTaken,
}

impl From<WasteError> for ServiceError {
    fn from(err: WasteError) -> Self {
        let code = match err {
            WasteError::Core(inner) => return inner,
            WasteError::NumberTaken => "waste_bin.number_taken",
        };
        ServiceError::Module {
            kind: ModuleErrorKind::Conflict,
            code,
            message: err.to_string(),
        }
    }
}

impl From<RepositoryError> for WasteError {
    fn from(e: RepositoryError) -> Self {
        Self::Core(ServiceError::Repository(e))
    }
}

impl From<ValidationError> for WasteError {
    fn from(e: ValidationError) -> Self {
        Self::Core(ServiceError::Validation(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_taken_number_is_a_conflict_with_its_own_code() {
        let err = ServiceError::from(WasteError::NumberTaken);
        assert!(matches!(
            err,
            ServiceError::Module {
                kind: ModuleErrorKind::Conflict,
                ..
            }
        ));
        assert_eq!(err.code(), "waste_bin.number_taken");
    }

    #[test]
    fn a_core_error_passes_through_unchanged() {
        let err = ServiceError::from(WasteError::from(ServiceError::InvalidInput("x".into())));
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }
}
