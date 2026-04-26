#[derive(Debug, serde::Serialize)]
pub struct PaginationRepsonse {
    pub total: u64,
    pub current_page: u64,
    pub total_pages: u64,
    pub next_page: Option<u64>,
    pub prev_page: Option<u64>,
}

impl PaginationRepsonse {
    pub fn new(total: u64, current_page: u64, total_pages: u64) -> Self {
        Self {
            total,
            current_page,
            total_pages,
            next_page: if current_page < total_pages {
                Some(current_page + 1)
            } else {
                None
            },
            prev_page: if current_page > 1 {
                Some(current_page - 1)
            } else {
                None
            },
        }
    }
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
