use domain::region::Region;

/// A geographic region used to group tree clusters.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({"id": 1, "name": "Stadtpark Flensburg"}))]
pub struct RegionResponse {
    /// Unique region identifier.
    #[schema(example = 1)]
    pub id: i32,

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
