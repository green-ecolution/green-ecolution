use serde_json::Value;

/// Provider attribution and provider-specific metadata for an entity.
///
/// `provider` distinguishes data sources
/// `additional_info` carries opaque provider payload.
#[derive(Debug, Default, Clone)]
pub struct ProviderInfo {
    pub provider: Option<String>,
    pub additional_info: Option<Value>,
}
