#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}
