use domain::shared::pagination::{DEFAULT_PAGE, DEFAULT_PER_PAGE, MAX_PER_PAGE, Pagination};

/// Pagination metadata returned in list responses.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({"total_records": 42, "current_page": 2, "per_page": 25, "total_pages": 5, "next_page": 3, "prev_page": 1}))]
pub struct PaginationResponse {
    /// Total number of records matching the query.
    #[schema(example = 42, minimum = 0)]
    pub total_records: u64,

    /// Current page number (1-based).
    #[schema(example = 2, minimum = 1)]
    pub current_page: u64,

    /// Page size used for this response.
    #[schema(example = 25, minimum = 1)]
    pub per_page: u64,

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
    pub fn new(total_records: u64, pagination: &Pagination) -> Self {
        let per_page = pagination.per_page();
        let current_page = pagination.page();
        let total_pages = total_records.div_ceil(per_page);

        // Cap current_page to total_pages so prev/next don't point to nonexistent pages.
        let effective_page = current_page.min(total_pages.max(1));

        let next_page = (effective_page < total_pages).then_some(effective_page + 1);
        let prev_page = (effective_page > 1 && total_pages > 0).then_some(effective_page - 1);

        Self {
            total_records,
            current_page,
            per_page,
            total_pages,
            next_page,
            prev_page,
        }
    }
}

/// Query parameters for paginated list endpoints.
///
/// Values outside the documented ranges are clamped server-side rather than rejected
/// (e.g. `per_page=10000` becomes `per_page=MAX_PER_PAGE`).
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

impl From<&PaginationParams> for Pagination {
    fn from(params: &PaginationParams) -> Self {
        Pagination::new(params.page, params.per_page)
    }
}

pub fn default_page() -> u64 {
    DEFAULT_PAGE
}

pub fn default_per_page() -> u64 {
    DEFAULT_PER_PAGE
}

// Surface MAX_PER_PAGE in module-local scope so it is referenced and kept in sync.
const _: u64 = MAX_PER_PAGE;

#[cfg(test)]
mod tests {
    use super::*;

    fn pagination(page: u64, per_page: u64) -> Pagination {
        Pagination::new(page, per_page)
    }

    #[test]
    fn empty_result_has_no_links() {
        let r = PaginationResponse::new(0, &pagination(1, 25));
        assert_eq!(r.total_records, 0);
        assert_eq!(r.total_pages, 0);
        assert_eq!(r.next_page, None);
        assert_eq!(r.prev_page, None);
    }

    #[test]
    fn first_page_has_no_prev() {
        let r = PaginationResponse::new(50, &pagination(1, 10));
        assert_eq!(r.total_pages, 5);
        assert_eq!(r.prev_page, None);
        assert_eq!(r.next_page, Some(2));
    }

    #[test]
    fn last_page_has_no_next() {
        let r = PaginationResponse::new(50, &pagination(5, 10));
        assert_eq!(r.next_page, None);
        assert_eq!(r.prev_page, Some(4));
    }

    #[test]
    fn out_of_range_page_clamps_links() {
        let r = PaginationResponse::new(5, &pagination(10, 10));
        assert_eq!(r.current_page, 10);
        assert_eq!(r.total_pages, 1);
        assert_eq!(r.next_page, None);
        assert_eq!(r.prev_page, None);
    }

    #[test]
    fn ceil_division_for_total_pages() {
        let r = PaginationResponse::new(11, &pagination(1, 10));
        assert_eq!(r.total_pages, 2);
    }
}
