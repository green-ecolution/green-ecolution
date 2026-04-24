use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    provider: String,
    additional_info: Value,
}

impl ProviderInfo {
    pub fn new(provider: String, additional_info: Value) -> Self {
        Self { provider, additional_info }
    }

    pub fn provider(&self) -> &str { &self.provider }
    pub fn additional_info(&self) -> &Value { &self.additional_info }
}
