use domain::region::Region;

/// A geographic region used to group tree clusters.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({"id": "0190a8e9-7c4f-7000-8000-000000000000", "name": "Stadtpark Flensburg"}))]
pub struct RegionResponse {
    /// Unique region identifier.
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,

    /// Human-readable name of the region.
    #[schema(example = "Stadtpark Flensburg")]
    pub name: String,
}

impl From<&Region> for RegionResponse {
    fn from(value: &Region) -> Self {
        Self {
            id: value.id.value(),
            name: value.name.as_str().to_string(),
        }
    }
}
