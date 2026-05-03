use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ValidationIssue {
    pub path: String,
    pub field: String,
    pub key: String,
    pub params: serde_json::Value,
}
