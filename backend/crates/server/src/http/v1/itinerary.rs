use std::sync::Arc;

use axum::extract::State;
use serde::Serialize;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use domain::{operation_plan::ItineraryEntry, planning::ResolvedStop};

use crate::{
    http::{
        AppState,
        auth::extractor::AuthUserExtractor,
        extractors::{Json, Path},
        v1::{dto::vehicle::CapacityKindDto, error::ErrorBody, plan::readable_plan},
    },
    service::ServiceError,
};

pub fn routes() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_plan_itinerary))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ItineraryKind {
    Depot,
    Stop,
    Station,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ItineraryEntryResponse {
    /// Position in driving order; the navigation endpoint addresses entries by it.
    pub index: usize,
    pub kind: ItineraryKind,
    /// Only for stops: the stop as the plan's module identifies it.
    #[schema(nullable)]
    pub stop: Option<Uuid>,
    /// Station or depot name, or the module's label of a stop. `null` for a stop
    /// its module no longer knows.
    #[schema(nullable)]
    pub name: Option<String>,
    #[schema(nullable)]
    pub lat: Option<f64>,
    #[schema(nullable)]
    pub lon: Option<f64>,
    /// Only for stops, in the unit of the module's capacity kind.
    #[schema(nullable)]
    pub demand: Option<f64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ItineraryResponse {
    pub plan_id: Uuid,
    pub module: String,
    /// `water` or `cargo_volume`.
    pub capacity_kind: CapacityKindDto,
    pub entries: Vec<ItineraryEntryResponse>,
}

pub fn build_entries(
    itinerary: &[ItineraryEntry],
    stops: &[ResolvedStop],
) -> Vec<ItineraryEntryResponse> {
    itinerary
        .iter()
        .enumerate()
        .map(|(index, entry)| match entry {
            ItineraryEntry::Depot(v) | ItineraryEntry::Station(v) => ItineraryEntryResponse {
                index,
                kind: if matches!(entry, ItineraryEntry::Depot(_)) {
                    ItineraryKind::Depot
                } else {
                    ItineraryKind::Station
                },
                stop: None,
                name: Some(v.name.as_str().to_owned()),
                lat: Some(v.coordinate.latitude()),
                lon: Some(v.coordinate.longitude()),
                demand: None,
            },
            ItineraryEntry::Stop(stop) => {
                let resolved = stops.iter().find(|s| s.stop == *stop);
                ItineraryEntryResponse {
                    index,
                    kind: ItineraryKind::Stop,
                    stop: Some(stop.value()),
                    name: resolved.map(|s| s.label.clone()),
                    lat: resolved.and_then(|s| s.location).map(|c| c.latitude()),
                    lon: resolved.and_then(|s| s.location).map(|c| c.longitude()),
                    demand: resolved.map(|s| s.demand),
                }
            }
        })
        .collect()
}

#[utoipa::path(get, path = "/plans/{plan_id}/itinerary", tag = "Plans",
    operation_id = "getPlanItinerary",
    summary = "Get the driving order of a plan",
    description = "Returns the persisted driving order (depot, stops, stations) with stop labels, positions and demand. A stop its module no longer knows keeps its place with null label and position.",
    params(("plan_id" = Uuid, Path, description = "Plan ID")),
    responses(
        (status = 200, description = "Driving order", body = ItineraryResponse),
        (status = 404, description = "Plan not found or has no route", body = ErrorBody),
        (status = 503, description = "Routing feature is disabled (code `feature.routing_disabled`)", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
#[tracing::instrument(level = "info", skip_all, fields(plan.id = %id))]
pub async fn get_plan_itinerary(
    State(state): State<Arc<AppState>>,
    user: AuthUserExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<ItineraryResponse>, ServiceError> {
    let plan = readable_plan(&state, user.caller(), id).await?;
    if plan.itinerary().is_empty() {
        return Err(domain::RepositoryError::NotFound.into());
    }
    let stops = state
        .operation_plan_service
        .resolved_stops_lenient(&plan)
        .await?;
    let capacity_kind = state.planning.provider(plan.module())?.capacity_kind();
    Ok(Json(ItineraryResponse {
        plan_id: id,
        module: plan.module().to_string(),
        capacity_kind: capacity_kind.into(),
        entries: build_entries(plan.itinerary(), &stops),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        operation_plan::StationVisit, planning::StopRef, shared::coordinates::Coordinate,
        start_point::StartPointName,
    };

    fn coord(lat: f64, lon: f64) -> Coordinate {
        Coordinate::new(lat, lon).unwrap()
    }

    fn visit(name: &str, lat: f64, lon: f64) -> StationVisit {
        StationVisit {
            name: StartPointName::new(name).unwrap(),
            coordinate: coord(lat, lon),
        }
    }

    #[test]
    fn entries_keep_driving_order_and_resolve_stops() {
        let stop = StopRef::new(uuid::Uuid::now_v7());
        let itinerary = vec![
            ItineraryEntry::Depot(visit("Hof", 54.0, 9.0)),
            ItineraryEntry::Stop(stop),
            ItineraryEntry::Station(visit("Hydrant", 54.2, 9.2)),
        ];
        let stops = vec![ResolvedStop {
            stop,
            label: "Cluster Nord".into(),
            location: Some(coord(54.1, 9.1)),
            demand: 640.0,
        }];

        let entries = build_entries(&itinerary, &stops);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].kind, ItineraryKind::Depot);
        assert_eq!(entries[0].name.as_deref(), Some("Hof"));
        assert_eq!(entries[1].kind, ItineraryKind::Stop);
        assert_eq!(entries[1].index, 1);
        assert_eq!(entries[1].stop, Some(stop.value()));
        assert_eq!(entries[1].name.as_deref(), Some("Cluster Nord"));
        assert_eq!(entries[1].lat, Some(54.1));
        assert_eq!(entries[1].demand, Some(640.0));
        assert_eq!(entries[2].kind, ItineraryKind::Station);
        assert_eq!(entries[2].lon, Some(9.2));
    }

    #[test]
    fn an_unresolved_stop_stays_in_place_without_label_and_position() {
        let stop = StopRef::new(uuid::Uuid::now_v7());
        let entries = build_entries(&[ItineraryEntry::Stop(stop)], &[]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].stop, Some(stop.value()));
        assert_eq!(entries[0].name, None);
        assert_eq!(entries[0].lat, None);
        assert_eq!(entries[0].lon, None);
        assert_eq!(entries[0].demand, None);
    }
}
