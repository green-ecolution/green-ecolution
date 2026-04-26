use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider: String,
    pub additional_info: Value,
}
