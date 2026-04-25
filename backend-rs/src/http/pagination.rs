#[derive(serde::Serialize)]
pub struct PaginationRepsonse {
    pub total: u64,
    pub current_page: u64,
    pub total_pages: u64,
    pub next_page: Option<u64>,
    pub prev_page: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    25
}
