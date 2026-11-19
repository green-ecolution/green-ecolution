use serde::{Deserialize, Serialize};

use domain::{
    settings::{
        DefectStreak, MapBounds, MapView, Patch, Resolution, SensorOfflineAfter, SettingsUpdate,
        ZoomLevel,
    },
    shared::{coordinates::Coordinate, error::ValidationError, geo::BoundingBox},
};

use crate::service::ServiceError;

/// One value as the settings screen needs it: what is in force, and what would
/// be in force without the organization's own value.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SettingField<T> {
    pub value: T,
    pub inherited_value: T,
}

pub fn field<T>(value: T, inherited_value: T) -> SettingField<T> {
    SettingField {
        value,
        inherited_value,
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone, Copy)]
pub struct MapViewDto {
    pub center: [f64; 2],
    /// `[sw_lat, sw_lng, ne_lat, ne_lng]`, or `null` where the map may be
    /// panned and zoomed freely. Absent reads the same as `null`: the generated
    /// client parses a null back into `undefined` and then omits the key
    /// entirely on the way out, so a round-tripped viewport must not be
    /// rejected.
    #[serde(default)]
    pub bbox: Option<[f64; 4]>,
    /// Set together with `bbox`, never on their own: the limits are one
    /// decision, and half of them would restrict a map for no stated reason.
    #[serde(default)]
    pub min_zoom: Option<u8>,
    #[serde(default)]
    pub max_zoom: Option<u8>,
}

impl From<MapView> for MapViewDto {
    fn from(m: MapView) -> Self {
        let bounds = m.bounds();
        Self {
            center: [m.center().latitude(), m.center().longitude()],
            bbox: bounds.map(|b| {
                let bbox = b.bbox();
                [bbox.sw_lat(), bbox.sw_lng(), bbox.ne_lat(), bbox.ne_lng()]
            }),
            min_zoom: bounds.map(|b| b.min_zoom().level()),
            max_zoom: bounds.map(|b| b.max_zoom().level()),
        }
    }
}

impl TryFrom<MapViewDto> for MapView {
    type Error = ServiceError;

    fn try_from(dto: MapViewDto) -> Result<Self, Self::Error> {
        let [lat, lng] = dto.center;
        let bounds = match (dto.bbox, dto.min_zoom, dto.max_zoom) {
            (None, None, None) => None,
            (Some([sw_lat, sw_lng, ne_lat, ne_lng]), Some(min), Some(max)) => Some(MapBounds::new(
                BoundingBox::try_new(sw_lat, sw_lng, ne_lat, ne_lng)?,
                ZoomLevel::new(min)?,
                ZoomLevel::new(max)?,
            )?),
            // Naming only part of the group is a caller error, not a viewport
            // with defaults quietly filled in around it.
            _ => {
                return Err(ServiceError::from(ValidationError::InvalidFormat {
                    field: "settings.map_view",
                    reason: "bbox, min_zoom and max_zoom are set together or not at all".into(),
                }));
            }
        };
        Ok(MapView::new(Coordinate::new(lat, lng)?, bounds)?)
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationSettingsResponse {
    pub sensor_offline_after_secs: SettingField<i64>,
    pub defect_streak: SettingField<i32>,
    pub map_view: SettingField<MapViewDto>,
}

/// An omitted field stays as it is, and so does `null`. Sending the value an
/// organization would inherit drops its own value.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OrganizationSettingsUpdateRequest {
    #[serde(default)]
    pub sensor_offline_after_secs: Option<i64>,
    #[serde(default)]
    pub defect_streak: Option<i32>,
    #[serde(default)]
    pub map_view: Option<MapViewDto>,
}

pub fn patch<T, U>(
    raw: Option<T>,
    build: impl FnOnce(T) -> Result<U, ServiceError>,
) -> Result<Patch<U>, ServiceError> {
    Ok(raw
        .map(build)
        .transpose()?
        .map_or(Patch::Unchanged, Patch::Set))
}

impl TryFrom<OrganizationSettingsUpdateRequest> for SettingsUpdate {
    type Error = ServiceError;

    fn try_from(req: OrganizationSettingsUpdateRequest) -> Result<Self, Self::Error> {
        Ok(SettingsUpdate {
            sensor_offline_after: patch(req.sensor_offline_after_secs, |v| {
                Ok(SensorOfflineAfter::new(v)?)
            })?,
            defect_streak: patch(req.defect_streak, |v| Ok(DefectStreak::new(v)?))?,
            map_view: patch(req.map_view, MapView::try_from)?,
        })
    }
}

impl OrganizationSettingsResponse {
    pub fn build(resolution: &Resolution) -> Self {
        let Resolution {
            effective,
            inherited,
        } = *resolution;
        Self {
            sensor_offline_after_secs: field(
                effective.sensor_offline_after.seconds(),
                inherited.sensor_offline_after.seconds(),
            ),
            defect_streak: field(
                effective.defect_streak.count(),
                inherited.defect_streak.count(),
            ),
            map_view: field(effective.map_view.into(), inherited.map_view.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_serializes_the_value_and_what_would_be_inherited() {
        let body = serde_json::to_value(field(120.0, 80.0)).unwrap();
        assert_eq!(
            body,
            serde_json::json!({ "value": 120.0, "inherited_value": 80.0 })
        );
    }
}
