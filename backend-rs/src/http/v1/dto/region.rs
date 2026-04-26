use crate::domain::region::Region;

#[derive(Debug, serde::Serialize)]
pub struct RegionResponse {
    pub id: i32,
    pub name: String,
}

impl From<&Region> for RegionResponse {
    fn from(value: &Region) -> Self {
        Self {
            id: value.id.value(),
            name: value.name.clone(),
        }
    }
}
