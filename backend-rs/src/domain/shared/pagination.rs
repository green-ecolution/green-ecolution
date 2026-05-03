/// Default page index when no `page` query parameter is provided.
pub const DEFAULT_PAGE: u64 = 1;
/// Default page size when no `per_page` query parameter is provided.
pub const DEFAULT_PER_PAGE: u64 = 25;
/// Maximum page size accepted from clients to bound query cost.
pub const MAX_PER_PAGE: u64 = 100;

/// A single page of results together with the total item count across all pages.
#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}

/// Validated pagination cursor.
///
/// `page` is clamped to `≥ 1`; `per_page` is clamped to `[1, MAX_PER_PAGE]`.
/// Callers that pass 0 or an oversized value are silently corrected rather
/// than getting an error, because these are usually benign client mistakes.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    page: u64,
    per_page: u64,
}

impl Pagination {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, MAX_PER_PAGE),
        }
    }

    pub fn page(&self) -> u64 {
        self.page
    }

    pub fn per_page(&self) -> u64 {
        self.per_page
    }

    pub fn limit(&self) -> u64 {
        self.per_page
    }

    pub fn offset(&self) -> u64 {
        (self.page - 1).saturating_mul(self.per_page)
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(DEFAULT_PAGE, DEFAULT_PER_PAGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_zero_page_to_one() {
        let p = Pagination::new(0, 10);
        assert_eq!(p.page(), 1);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn clamps_zero_per_page_to_one() {
        let p = Pagination::new(1, 0);
        assert_eq!(p.per_page(), 1);
    }

    #[test]
    fn clamps_oversized_per_page_to_max() {
        let p = Pagination::new(1, 10_000);
        assert_eq!(p.per_page(), MAX_PER_PAGE);
    }

    #[test]
    fn computes_offset() {
        let p = Pagination::new(3, 25);
        assert_eq!(p.offset(), 50);
    }
}
