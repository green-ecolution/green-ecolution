use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::{
    Id,
    organization::Organization,
    shared::{coordinates::Coordinate, error::ValidationError, provenance::Provenance},
};
use waste_domain::{
    fill::FillState,
    values::{BinDepth, BinVolume, WasteBinNumber},
    waste_bin::{WasteBinDraft, WasteBinUpdate},
};

use crate::service::waste_bin_service::WasteBinState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FillStateDto {
    Empty,
    Normal,
    Full,
    Unknown,
}

impl From<FillState> for FillStateDto {
    fn from(state: FillState) -> Self {
        match state {
            FillState::Empty => Self::Empty,
            FillState::Normal => Self::Normal,
            FillState::Full => Self::Full,
            FillState::Unknown => Self::Unknown,
        }
    }
}

impl From<FillStateDto> for FillState {
    fn from(state: FillStateDto) -> Self {
        match state {
            FillStateDto::Empty => Self::Empty,
            FillStateDto::Normal => Self::Normal,
            FillStateDto::Full => Self::Full,
            FillStateDto::Unknown => Self::Unknown,
        }
    }
}

/// A waste container with its current fill level.
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WasteBinResponse {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub id: Uuid,
    #[schema(example = "K-12")]
    pub number: String,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = 1100.0)]
    pub volume_liters: f64,
    /// Inner height; the distance a sensor reads when the bin is empty.
    #[schema(example = 120)]
    pub depth_cm: i32,
    pub organization_id: Uuid,
    #[schema(example = "green-ecolution", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Last measured fill level, rounded to whole percent.
    #[schema(example = 75, minimum = 0, maximum = 100, nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_percent: Option<i32>,
    #[schema(example = "2026-11-02T08:00:00+00:00", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_measured_at: Option<String>,
    #[schema(example = "2026-11-02T09:30:00+00:00", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_emptied_at: Option<String>,
    pub fill_state: FillStateDto,
    #[schema(example = "eui-a81758fffe0c3b52", nullable)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensor_id: Option<String>,
    #[schema(example = "2026-10-27T10:30:00+00:00")]
    pub created_at: String,
    #[schema(example = "2026-10-27T10:30:00+00:00")]
    pub updated_at: String,
}

impl From<&WasteBinState> for WasteBinResponse {
    fn from(state: &WasteBinState) -> Self {
        let bin = &state.view;
        Self {
            id: bin.id.value(),
            number: bin.number.as_str().to_owned(),
            latitude: bin.coordinate.latitude(),
            longitude: bin.coordinate.longitude(),
            volume_liters: bin.volume.liters(),
            depth_cm: bin.depth.centimeters() as i32,
            organization_id: bin.organization_id.value(),
            provider: bin.provenance.provider().map(|p| p.as_str().to_owned()),
            fill_percent: bin.fill.map(|f| (f.fraction * 100.0).round() as i32),
            fill_measured_at: bin.fill.map(|f| f.measured_at.to_rfc3339()),
            last_emptied_at: bin.last_emptied_at.map(|at| at.to_rfc3339()),
            fill_state: state.fill_state.into(),
            sensor_id: bin.sensor_id.as_ref().map(|s| s.as_str().to_owned()),
            created_at: bin.created_at.to_rfc3339(),
            updated_at: bin.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WasteBinCreateRequest {
    #[schema(example = "K-12")]
    pub number: String,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = 1100.0, minimum = 1.0, maximum = 100000.0)]
    pub volume_liters: f64,
    #[schema(example = 120, minimum = 1, maximum = 1000)]
    pub depth_cm: i32,
    /// Defaults to the caller's own organization.
    #[schema(nullable)]
    #[serde(default)]
    pub organization_id: Option<Uuid>,
}

impl WasteBinCreateRequest {
    pub fn into_draft(self, org: Id<Organization>) -> Result<WasteBinDraft, ValidationError> {
        Ok(WasteBinDraft {
            number: WasteBinNumber::new(self.number)?,
            coordinate: Coordinate::new(self.latitude, self.longitude)?,
            volume: BinVolume::new(self.volume_liters)?,
            depth: BinDepth::new(i64::from(self.depth_cm))?,
            organization_id: org,
            provenance: Provenance::default(),
        })
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WasteBinUpdateRequest {
    #[schema(example = "K-12")]
    pub number: String,
    #[schema(example = 54.7937, minimum = -90.0, maximum = 90.0)]
    pub latitude: f64,
    #[schema(example = 9.4469, minimum = -180.0, maximum = 180.0)]
    pub longitude: f64,
    #[schema(example = 1100.0, minimum = 1.0, maximum = 100000.0)]
    pub volume_liters: f64,
    #[schema(example = 120, minimum = 1, maximum = 1000)]
    pub depth_cm: i32,
}

impl TryFrom<WasteBinUpdateRequest> for WasteBinUpdate {
    type Error = ValidationError;

    fn try_from(req: WasteBinUpdateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            number: WasteBinNumber::new(req.number)?,
            coordinate: Coordinate::new(req.latitude, req.longitude)?,
            volume: BinVolume::new(req.volume_liters)?,
            depth: BinDepth::new(i64::from(req.depth_cm))?,
        })
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WasteBinTransferRequest {
    #[schema(example = "0190a8e9-7c4f-7000-8000-000000000000")]
    pub organization_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct WasteBinListParams {
    #[param(default = 1, minimum = 1, example = 1)]
    #[serde(default = "server::http::v1::pagination::default_page")]
    pub page: u64,
    #[param(default = 25, minimum = 1, maximum = 100, example = 25)]
    #[serde(default = "server::http::v1::pagination::default_per_page")]
    pub per_page: u64,
    /// Case-insensitive substring of the bin number.
    #[param(example = "K-1")]
    pub number: Option<String>,
    pub organization_id: Option<Uuid>,
    /// Judged by the threshold and offline period of `organizationId`, or of
    /// the caller's organization when none is given.
    pub fill_state: Option<FillStateDto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use waste_domain::{fill::FillMeasurement, waste_bin::WasteBinView};

    fn state(fraction: Option<f64>) -> WasteBinState {
        WasteBinState {
            view: WasteBinView {
                id: Id::new_v7(),
                number: WasteBinNumber::new("K-1").unwrap(),
                coordinate: Coordinate::new(54.79, 9.44).unwrap(),
                volume: BinVolume::new(1100.0).unwrap(),
                depth: BinDepth::new(120).unwrap(),
                organization_id: Id::new_v7(),
                provenance: Provenance::default(),
                fill: fraction.map(|fraction| FillMeasurement {
                    fraction,
                    measured_at: Utc.with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap(),
                }),
                last_emptied_at: None,
                sensor_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            fill_state: FillState::Normal,
        }
    }

    #[test]
    fn the_fill_level_is_rounded_to_whole_percent() {
        assert_eq!(
            WasteBinResponse::from(&state(Some(0.756))).fill_percent,
            Some(76)
        );
        assert_eq!(WasteBinResponse::from(&state(None)).fill_percent, None);
    }

    #[test]
    fn fields_are_camel_case() {
        let json = serde_json::to_value(WasteBinResponse::from(&state(Some(0.5)))).unwrap();
        for key in [
            "volumeLiters",
            "depthCm",
            "organizationId",
            "fillPercent",
            "fillState",
        ] {
            assert!(json.get(key).is_some(), "{key} missing in {json}");
        }
        assert_eq!(json["fillState"], "normal");
    }
}
