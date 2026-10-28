use std::sync::Arc;

use server::service::{
    authorization::AuthorizationService, module_access::ModuleAccess, user_service::UserService,
};

use crate::service::waste_bin_service::WasteBinService;

pub struct WasteHttpState {
    pub bins: Arc<WasteBinService>,
    pub access: ModuleAccess,
    pub authorization: Arc<AuthorizationService>,
    pub users: Arc<UserService>,
}
