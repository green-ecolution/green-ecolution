use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use domain::{
    Id,
    organization::Organization,
    settings::{
        DefectStreak, JustWateredTtl, MapView, OrganizationSettings, Patch, Resolution,
        SensorOfflineAfter, SettingChangeEntry, SettingKey, SettingOrigin, SettingsUpdate,
        WaterDemand,
    },
    shared::{coordinates::Coordinate, geo::BoundingBox},
};

use crate::service::ServiceError;

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettingOriginDto {
    Default,
    Inherited,
    Own,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationRef {
    pub id: Uuid,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SettingChangeRef {
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub changed_by: Option<Uuid>,
}

/// One value plus everything the interface needs to show where it comes from
/// and to offer "set my own" or "inherit again".
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SettingField<T> {
    /// The value in force.
    pub value: T,
    pub origin: SettingOriginDto,
    /// Set only where `origin` is `inherited`.
    pub source: Option<OrganizationRef>,
    /// This organization's own value, reported even while a lock above makes
    /// it dormant — a value that silently disappears and reappears on unlock
    /// is the first thing someone reports as a bug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_value: Option<T>,
    /// Always about this organization's own value, never the source's.
    pub last_change: Option<SettingChangeRef>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone, Copy)]
pub struct MapViewDto {
    pub center: [f64; 2],
    /// `[sw_lat, sw_lng, ne_lat, ne_lng]`.
    pub bbox: [f64; 4],
}

impl From<MapView> for MapViewDto {
    fn from(m: MapView) -> Self {
        Self {
            center: [m.center().latitude(), m.center().longitude()],
            bbox: [
                m.bbox().sw_lat(),
                m.bbox().sw_lng(),
                m.bbox().ne_lat(),
                m.bbox().ne_lng(),
            ],
        }
    }
}

impl TryFrom<MapViewDto> for MapView {
    type Error = ServiceError;

    fn try_from(dto: MapViewDto) -> Result<Self, Self::Error> {
        let [lat, lng] = dto.center;
        let [sw_lat, sw_lng, ne_lat, ne_lng] = dto.bbox;
        Ok(MapView::new(
            Coordinate::new(lat, lng)?,
            BoundingBox::try_new(sw_lat, sw_lng, ne_lat, ne_lng)?,
        )?)
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationSettingsResponse {
    pub water_demand: SettingField<f64>,
    pub just_watered_ttl_secs: SettingField<i64>,
    pub sensor_offline_after_secs: SettingField<i64>,
    pub defect_streak: SettingField<i32>,
    pub map_view: SettingField<MapViewDto>,
    /// This organization's own switch for its sub-units. While `enforced_by`
    /// is set, an organization above has frozen the subtree and no sub-unit
    /// may set anything, whatever this says.
    pub descendants_may_override: bool,
    /// The organization that froze this subtree, if any.
    pub enforced_by: Option<OrganizationRef>,
}

/// Three states per field. Absent means unchanged, `null` means go back to
/// inheriting, a value means set it.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OrganizationSettingsUpdateRequest {
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub water_demand: Option<Option<f64>>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<i64>)]
    pub just_watered_ttl_secs: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<i64>)]
    pub sensor_offline_after_secs: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<i32>)]
    pub defect_streak: Option<Option<i32>>,
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<MapViewDto>)]
    pub map_view: Option<Option<MapViewDto>>,
    /// A plain toggle, not a three-state field: `null` is equivalent to
    /// omitting it and leaves the switch as it is.
    #[serde(default)]
    pub descendants_may_override: Option<bool>,
}

/// `Option<Option<T>>` needs an explicit deserializer: serde's `default`
/// collapses "absent" and "null" into `None` otherwise, and every partial save
/// would reset the fields it did not send.
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer).map(Some)
}

fn patch<T, U, F>(raw: Option<Option<T>>, build: F) -> Result<Patch<U>, ServiceError>
where
    F: FnOnce(T) -> Result<U, ServiceError>,
{
    match raw {
        None => Ok(Patch::Unchanged),
        Some(None) => Ok(Patch::Clear),
        Some(Some(value)) => Ok(Patch::Set(build(value)?)),
    }
}

impl TryFrom<OrganizationSettingsUpdateRequest> for SettingsUpdate {
    type Error = ServiceError;

    fn try_from(req: OrganizationSettingsUpdateRequest) -> Result<Self, Self::Error> {
        Ok(SettingsUpdate {
            water_demand: patch(req.water_demand, |v| Ok(WaterDemand::new(v)?))?,
            just_watered_ttl: patch(req.just_watered_ttl_secs, |v| Ok(JustWateredTtl::new(v)?))?,
            sensor_offline_after: patch(req.sensor_offline_after_secs, |v| {
                Ok(SensorOfflineAfter::new(v)?)
            })?,
            defect_streak: patch(req.defect_streak, |v| Ok(DefectStreak::new(v)?))?,
            map_view: patch(req.map_view, MapView::try_from)?,
            descendants_may_override: req.descendants_may_override,
        })
    }
}

fn field<T>(
    value: T,
    origin: SettingOrigin,
    own: Option<T>,
    last: Option<&SettingChangeEntry>,
) -> SettingField<T> {
    SettingField {
        value,
        origin: match origin {
            SettingOrigin::Default => SettingOriginDto::Default,
            SettingOrigin::Inherited(_) => SettingOriginDto::Inherited,
            SettingOrigin::Own => SettingOriginDto::Own,
        },
        source: match origin {
            SettingOrigin::Inherited(id) => Some(OrganizationRef { id: id.value() }),
            _ => None,
        },
        own_value: own,
        last_change: last.map(|e| SettingChangeRef {
            changed_at: e.changed_at,
            changed_by: e.changed_by,
        }),
    }
}

impl OrganizationSettingsResponse {
    pub fn build(
        resolution: &Resolution,
        own: &OrganizationSettings,
        last: &std::collections::HashMap<SettingKey, SettingChangeEntry>,
    ) -> Self {
        let effective = resolution.effective;
        let origins = resolution.origins;
        Self {
            water_demand: field(
                effective.water_demand.liters(),
                origins.water_demand,
                own.water_demand.map(|v| v.liters()),
                last.get(&SettingKey::WaterDemand),
            ),
            just_watered_ttl_secs: field(
                effective.just_watered_ttl.seconds(),
                origins.just_watered_ttl,
                own.just_watered_ttl.map(|v| v.seconds()),
                last.get(&SettingKey::JustWateredTtl),
            ),
            sensor_offline_after_secs: field(
                effective.sensor_offline_after.seconds(),
                origins.sensor_offline_after,
                own.sensor_offline_after.map(|v| v.seconds()),
                last.get(&SettingKey::SensorOfflineAfter),
            ),
            defect_streak: field(
                effective.defect_streak.count(),
                origins.defect_streak,
                own.defect_streak.map(|v| v.count()),
                last.get(&SettingKey::DefectStreak),
            ),
            map_view: field(
                effective.map_view.into(),
                origins.map_view,
                own.map_view.map(MapViewDto::from),
                last.get(&SettingKey::MapView),
            ),
            descendants_may_override: own.descendants_may_override,
            enforced_by: effective
                .enforced_by
                .map(|id: Id<Organization>| OrganizationRef { id: id.value() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// utoipa cannot carry `Option`'s nullability through a generic argument,
    /// so the schema describes `own_value` as optional and not nullable. The
    /// body has to match that: omit the key rather than send `null`.
    #[test]
    fn a_field_without_an_own_value_omits_the_key() {
        let field = SettingField {
            value: 80.0,
            origin: SettingOriginDto::Default,
            source: None,
            own_value: None,
            last_change: None,
        };
        let body = serde_json::to_value(&field).unwrap();
        assert!(!body.as_object().unwrap().contains_key("own_value"));
    }
}
