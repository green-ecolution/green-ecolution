use crate::service::AuthError;
use domain::{auth::AuthUser, user::UserRole};

pub fn require_any_role(user: &AuthUser, roles: &[UserRole]) -> Result<(), AuthError> {
    if user.roles.iter().any(|r| roles.contains(r)) {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

#[macro_export]
macro_rules! require_role {
    ($user:expr, $($role:expr),+ $(,)?) => {
        $crate::http::auth::roles::require_any_role(
            &$user,
            &[$($role),+],
        )?
    };
}
