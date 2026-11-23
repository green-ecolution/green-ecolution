use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use domain::auth::AuthUser;

use crate::service::{AuthError, module_access::ModuleAccess};

/// Runs inside the authentication layer, in front of every protected route
/// of one module. A module locked for the caller's tenant answers as if it
/// were switched off, on every route, including ones that check nothing
/// themselves.
pub async fn module_guard(
    State(access): State<ModuleAccess>,
    request: Request,
    next: Next,
) -> Response {
    let Some(user) = request.extensions().get::<AuthUser>().cloned() else {
        return AuthError::MissingToken.into_response();
    };
    match access.ensure_released(user.caller()).await {
        Ok(()) => next.run(request).await,
        Err(error) => error.into_response(),
    }
}
