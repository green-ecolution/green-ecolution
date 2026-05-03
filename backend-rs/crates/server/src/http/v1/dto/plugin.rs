use serde::{Deserialize, Serialize};

use domain::plugin::Plugin;

use super::user::ClientTokenResponse;

/// Represents a registered plugin in the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "weather-provider",
    "name": "Weather Provider Plugin",
    "host_path": "https://plugin.example.com",
    "version": "1.0.0",
    "description": "Provides weather forecast data"
}))]
pub struct PluginResponse {
    /// Unique slug identifier for the plugin.
    #[schema(example = "weather-provider")]
    pub slug: String,
    /// Human-readable plugin name.
    #[schema(example = "Weather Provider Plugin")]
    pub name: String,
    /// Base URL where the plugin is hosted.
    #[schema(example = "https://plugin.example.com")]
    pub host_path: String,
    /// Semantic version of the plugin.
    #[schema(example = "1.0.0")]
    pub version: String,
    /// Short description of the plugin's purpose.
    #[schema(example = "Provides weather forecast data")]
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

/// Authentication credentials for a plugin's OIDC client.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PluginAuthRequest {
    /// OIDC client ID assigned to the plugin.
    #[schema(example = "weather-provider-client")]
    pub client_id: String,
    /// OIDC client secret for authentication.
    #[schema(example = "s3cr3t-k3y")]
    pub client_secret: String,
}

/// Request body for registering a new plugin.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "slug": "weather-provider",
    "name": "Weather Provider Plugin",
    "path": "https://plugin.example.com",
    "version": "1.0.0",
    "description": "Provides weather forecast data",
    "auth": {
        "client_id": "weather-provider-client",
        "client_secret": "s3cr3t-k3y"
    }
}))]
pub struct PluginRegisterRequest {
    /// Unique slug identifier for the plugin.
    #[schema(example = "weather-provider")]
    pub slug: String,
    /// Human-readable plugin name.
    #[schema(example = "Weather Provider Plugin")]
    pub name: String,
    /// Base URL where the plugin is hosted.
    #[schema(example = "https://plugin.example.com")]
    pub path: String,
    /// Semantic version of the plugin.
    #[schema(example = "1.0.0")]
    pub version: String,
    /// Short description of the plugin's purpose.
    #[schema(example = "Provides weather forecast data")]
    pub description: String,
    /// Authentication credentials for the plugin.
    pub auth: PluginAuthRequest,
}

pub type PluginRegisterResponse = ClientTokenResponse;

/// Response containing a list of registered plugins.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PluginListResponse {
    /// List of registered plugins.
    pub data: Vec<PluginResponse>,
}
