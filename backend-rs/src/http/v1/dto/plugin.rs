use serde::{Deserialize, Serialize};

use crate::domain::plugin::Plugin;

use super::user::ClientTokenResponse;

#[derive(Debug, Serialize)]
pub struct PluginResponse {
    pub slug: String,
    pub name: String,
    pub host_path: String,
    pub version: String,
    pub description: String,
}

impl From<&Plugin> for PluginResponse {
    fn from(value: &Plugin) -> Self {
        Self {
            slug: value.slug.clone(),
            name: value.name.clone(),
            host_path: value.path.to_string(),
            version: value.version.clone(),
            description: value.description.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PluginListResponse {
    pub plugins: Vec<PluginResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginAuthRequest {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct PluginRegisterRequest {
    pub slug: String,
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
    pub auth: PluginAuthRequest,
}

pub type PluginRegisterResponse = ClientTokenResponse;
