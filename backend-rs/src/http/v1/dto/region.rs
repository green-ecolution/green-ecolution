use crate::{
    domain::{region::Region, shared::pagination::Page},
    http::v1::pagination::PaginationRepsonse,
};

#[derive(Debug, serde::Serialize)]
pub struct RegionResponse {
    pub id: i32,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct RegionListResponse {
    pub data: Vec<RegionResponse>,
    pub pagination: PaginationRepsonse,
}

impl RegionListResponse {
    pub fn from_page(page: Page<Region>, current_page: u64, per_page: u64) -> Self {
        let total_pages = (page.total + per_page - 1) / per_page;
        Self {
            data: page.items.iter().map(RegionResponse::from).collect(),
            pagination: PaginationRepsonse {
                total: page.total,
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
            },
        }
    }
}

impl From<&Region> for RegionResponse {
    fn from(value: &Region) -> Self {
        Self {
            id: value.id.value(),
            name: value.name.clone(),
        }
    }
}
