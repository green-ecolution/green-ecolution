/// Pagination metadata returned in list responses.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({"total_records": 42, "current_page": 2, "total_pages": 5, "next_page": 3, "prev_page": 1}))]
pub struct PaginationResponse {
    /// Total number of records matching the query.
    #[schema(example = 42, minimum = 0)]
    pub total_records: u64,

    /// Current page number (1-based).
    #[schema(example = 2, minimum = 1)]
    pub current_page: u64,

    /// Total number of pages available.
    #[schema(example = 5, minimum = 0)]
    pub total_pages: u64,

    /// Next page number, if available.
    #[schema(example = 3, nullable)]
    pub next_page: Option<u64>,

    /// Previous page number, if available.
    #[schema(example = 1, nullable)]
    pub prev_page: Option<u64>,
}

impl PaginationResponse {
    pub fn new(total_records: u64, current_page: u64, total_pages: u64) -> Self {
        Self {
            total_records,
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

/// Query parameters for paginated list endpoints.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct PaginationParams {
    /// Page number to retrieve (1-based).
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "default_page")]
    pub page: u64,

    /// Number of items per page.
    #[param(default = 25, minimum = 1, maximum = 100, example = 25)]
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    25
}
