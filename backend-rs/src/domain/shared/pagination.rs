#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: u64,
    pub offset: u64,
}

impl Pagination {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            limit: per_page,
            offset: page.saturating_sub(1) * per_page,
        }
    }
}
