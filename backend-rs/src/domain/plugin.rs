//! Plugin manifest shapes — describes a registered external plugin with its
//! location and OAuth client credentials.

use url::Url;

#[derive(Debug, Clone)]
pub struct AuthPlugin {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone)]
pub struct Plugin {
    pub slug: String,
    pub name: String,
    pub path: Url,
    pub version: String,
    pub description: String,
    pub auth: AuthPlugin,
}
