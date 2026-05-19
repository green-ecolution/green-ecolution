use serde::{Deserialize, Deserializer, Serialize};

use domain::{
    Id,
    sensor::{SensorId, SensorView},
    shared::{
        coordinates::Coordinate,
        error::ValidationError,
        geo::BoundingBox,
        provenance::{Provenance, ProviderId},
    },
    tree::{PlantingYear, Species, TreeDraft, TreeMarker, TreeNumber, TreeView},
};

use super::{WateringStatus, sensor::SensorResponse};

// serde_urlencoded (used by axum's Query extractor) does not support repeated keys
// into Vec<T>. This deserializer accepts a single scalar ("2020") or null.
fn deserialize_optional_vec_i32<'de, D>(deserializer: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct OptVecI32Visitor;

    impl<'de> Visitor<'de> for OptVecI32Visitor {
        type Value = Option<Vec<i32>>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an integer, a string integer, or null")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_any(InnerVisitor).map(Some)
        }

        fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(vec![v as i32]))
        }

        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(vec![v as i32]))
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let n = v.parse::<i32>().map_err(E::custom)?;
            Ok(Some(vec![n]))
        }
    }

    struct InnerVisitor;

    impl<'de> Visitor<'de> for InnerVisitor {
        type Value = Vec<i32>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an integer or a string integer")
        }

        fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(vec![v as i32])
        }

        fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(vec![v as i32])
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let n = v.parse::<i32>().map_err(E::custom)?;
            Ok(vec![n])
        }
    }

    deserializer.deserialize_option(OptVecI32Visitor)
}

// serde_urlencoded (used by axum's Query extractor) does not support repeated keys
// into Vec<T>. This deserializer accepts a single scalar ("good") or null.
fn deserialize_optional_vec_watering_status<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<super::WateringStatus>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, IntoDeserializer, Visitor};
    use std::fmt;

    use super::WateringStatus;

    struct OptVecVisitor;

    impl<'de> Visitor<'de> for OptVecVisitor {
        type Value = Option<Vec<WateringStatus>>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a watering_status string or null")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_any(InnerVisitor).map(Some)
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let item = WateringStatus::deserialize(v.into_deserializer())?;
            Ok(Some(vec![item]))
        }
    }

    struct InnerVisitor;

    impl<'de> Visitor<'de> for InnerVisitor {
        type Value = Vec<WateringStatus>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a watering_status string")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let item = WateringStatus::deserialize(v.into_deserializer())?;
            Ok(vec![item])
        }
    }

    deserializer.deserialize_option(OptVecVisitor)
}

/// An individual tree managed by the system.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = "2024-01-15T10:30:00+00:00")]
    pub created_at: String,
    #[schema(example = "2024-06-20T14:00:00+00:00")]
    pub updated_at: String,
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    pub watering_status: WateringStatus,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_cluster_id: Option<uuid::Uuid>,
    #[schema(nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor: Option<SensorResponse>,
    #[schema(example = "2024-07-10T08:00:00+00:00", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<serde_json::Value>,
}

impl From<(&TreeView, Option<&SensorView>)> for TreeResponse {
    fn from((tree, sensor): (&TreeView, Option<&SensorView>)) -> Self {
        Self {
            id: tree.id,
            created_at: tree.created_at.to_rfc3339(),
            updated_at: tree.updated_at.to_rfc3339(),
            species: tree.species.clone(),
            number: tree.tree_number.clone(),
            planting_year: tree.planting_year as i32,
            latitude: tree.latitude,
            longitude: tree.longitude,
            watering_status: tree.watering_status.into(),
            description: tree.description.clone().unwrap_or_default(),
            tree_cluster_id: tree.cluster_id,
            sensor: sensor.map(SensorResponse::from),
            last_watered: tree.last_watered.map(|dt| dt.to_rfc3339()),
            provider: tree.provider.clone(),
            additional_information: tree.additional_info.clone(),
        }
    }
}

/// A tree with its distance from a reference point.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeWithDistanceResponse {
    pub tree: TreeResponse,
    #[schema(example = 142.5, minimum = 0.0)]
    pub distance_meters: f64,
}

/// Query parameters for finding the nearest trees to a given point.
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct NearestTreeParams {
    #[param(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub lat: f64,
    #[param(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub lng: f64,
    #[param(example = 10, minimum = 1, maximum = 100)]
    #[serde(default)]
    pub limit: Option<u64>,
}

/// Non-paginated list of nearest trees with distance information.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NearestTreeListResponse {
    pub data: Vec<TreeWithDistanceResponse>,
}

/// Request body for creating a new tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeCreateRequest {
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<uuid::Uuid>,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

impl TryFrom<TreeCreateRequest> for TreeDraft {
    type Error = ValidationError;

    fn try_from(req: TreeCreateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            cluster_id: req.tree_cluster_id.map(Id::new),
            sensor_id: req.sensor_id.map(SensorId::new).transpose()?,
            planting_year: PlantingYear::new(req.planting_year as u32)?,
            species: Species::new(req.species)?,
            tree_number: TreeNumber::new(req.number)?,
            coordinate: Coordinate::new(req.latitude, req.longitude)?,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            provenance: Provenance::new(
                req.provider.map(ProviderId::new).transpose()?,
                req.additional_information,
            ),
        })
    }
}

/// Request body for updating an existing tree.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TreeUpdateRequest {
    #[schema(example = "Quercus robur")]
    pub species: String,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    #[schema(example = 2020, minimum = 1900, maximum = 2100)]
    pub planting_year: i32,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = "Standort nahe Spielplatz")]
    pub description: String,
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000", nullable)]
    #[serde(default)]
    pub tree_cluster_id: Option<uuid::Uuid>,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(default)]
    pub sensor_id: Option<String>,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(default)]
    pub provider: Option<String>,
    #[schema(value_type = Object, nullable)]
    #[serde(default)]
    pub additional_information: Option<serde_json::Value>,
}

/// Lightweight tree marker for the map.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeMarkerResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: uuid::Uuid,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    pub watering_status: WateringStatus,
    #[schema(example = "T-2024-0042")]
    pub number: String,
    pub has_sensor: bool,
}

impl From<&TreeMarker> for TreeMarkerResponse {
    fn from(m: &TreeMarker) -> Self {
        Self {
            id: m.id,
            latitude: m.latitude,
            longitude: m.longitude,
            watering_status: m.watering_status.into(),
            number: m.tree_number.clone(),
            has_sensor: m.has_sensor,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TreeMarkerListResponse {
    pub data: Vec<TreeMarkerResponse>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct TreeMarkerQueryParams {
    /// Format: `swLat,swLng,neLat,neLng`. Required.
    #[param(example = "54.78,9.40,54.81,9.46")]
    pub bbox: String,
    #[param(nullable)]
    #[serde(default)]
    pub has_cluster: Option<bool>,
    #[param(nullable)]
    // Single value only; repeated keys are not collected by serde_urlencoded.
    #[serde(default, deserialize_with = "deserialize_optional_vec_i32")]
    pub planting_year: Option<Vec<i32>>,
    #[param(nullable)]
    // Single value only; repeated keys are not collected by serde_urlencoded.
    #[serde(default, deserialize_with = "deserialize_optional_vec_watering_status")]
    pub watering_status: Option<Vec<WateringStatus>>,
}

impl TreeMarkerQueryParams {
    pub fn parse_bbox(&self) -> Result<BoundingBox, String> {
        let parts: Vec<&str> = self.bbox.split(',').collect();
        if parts.len() != 4 {
            return Err(format!(
                "bbox must be 'swLat,swLng,neLat,neLng' (got {} parts)",
                parts.len()
            ));
        }
        let sw_lat: f64 = parts[0]
            .trim()
            .parse()
            .map_err(|e| format!("sw_lat: {e}"))?;
        let sw_lng: f64 = parts[1]
            .trim()
            .parse()
            .map_err(|e| format!("sw_lng: {e}"))?;
        let ne_lat: f64 = parts[2]
            .trim()
            .parse()
            .map_err(|e| format!("ne_lat: {e}"))?;
        let ne_lng: f64 = parts[3]
            .trim()
            .parse()
            .map_err(|e| format!("ne_lng: {e}"))?;
        BoundingBox::try_new(sw_lat, sw_lng, ne_lat, ne_lng).map_err(|e| e.to_string())
    }
}
