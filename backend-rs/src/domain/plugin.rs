use url::Url;

#[derive(Debug, Clone)]
pub struct AuthPlugin {
    client_id: String,
    client_secret: String,
}

impl AuthPlugin {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self { client_id, client_secret }
    }

    pub fn client_id(&self) -> &str { &self.client_id }
    pub fn client_secret(&self) -> &str { &self.client_secret }
}

#[derive(Debug, Clone)]
pub struct Plugin {
    slug: String,
    name: String,
    path: Url,
    version: String,
    description: String,
    auth: AuthPlugin,
}

impl Plugin {
    pub fn new(
        slug: String,
        name: String,
        path: Url,
        version: String,
        description: String,
        auth: AuthPlugin,
    ) -> Self {
        Self { slug, name, path, version, description, auth }
    }

    pub fn slug(&self) -> &str { &self.slug }
    pub fn name(&self) -> &str { &self.name }
    pub fn path(&self) -> &Url { &self.path }
    pub fn version(&self) -> &str { &self.version }
    pub fn description(&self) -> &str { &self.description }
    pub fn auth(&self) -> &AuthPlugin { &self.auth }
}
