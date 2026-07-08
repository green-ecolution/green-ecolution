use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use domain::{
    Id,
    cluster::TreeCluster,
    routing::{OptimizedRoute, RouteOptimizer, RouteStop, RoutingError},
    shared::{coordinates::Coordinate, distance::Distance},
    vehicle::Vehicle,
};

use crate::configuration::RoutingSettings;

// Parity with the retired Go/Vroom setup: full-day shift, zero service and
// refill durations keep the solver time-unconstrained.
const SHIFT_END_SECS: f64 = 86_400.0;
const SERVICE_TIME_SECS: f64 = 0.0;
const REFILL_DURATION_SECS: f64 = 0.0;

pub struct StreamletRouteOptimizer {
    client: reqwest::Client,
    solve_url: String,
    default_depot: LocationDto,
    refills: Vec<LocationDto>,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct LocationDto {
    lat: f64,
    lon: f64,
}

#[derive(Serialize)]
struct TankDto {
    capacity: f64,
    level: f64,
}

#[derive(Serialize)]
enum KindDto {
    Truck {
        width: f64,
        height: f64,
        length: f64,
        weight: f64,
    },
}

#[derive(Serialize)]
struct ShiftDto {
    start: f64,
    end: f64,
}

#[derive(Serialize)]
struct VehicleDto {
    id: u32,
    start: LocationDto,
    tank: TankDto,
    kind: KindDto,
    shift: ShiftDto,
    max_trips: Option<u32>,
}

#[derive(Serialize)]
struct DepotDto {
    id: u32,
    location: LocationDto,
}

#[derive(Serialize)]
struct CustomerDto {
    id: u32,
    location: LocationDto,
    demand: f64,
    service_time: f64,
    time_window: Option<ShiftDto>,
}

#[derive(Serialize)]
struct RefillStationDto {
    id: u32,
    location: LocationDto,
    refill_duration: f64,
}

#[derive(Serialize)]
struct ProblemDto {
    vehicles: Vec<VehicleDto>,
    depots: Vec<DepotDto>,
    customers: Vec<CustomerDto>,
    refill_stations: Vec<RefillStationDto>,
}

#[derive(Serialize)]
struct OptionsDto {
    geometry: &'static str,
}

#[derive(Serialize)]
struct SolveRequestDto {
    problem: ProblemDto,
    options: OptionsDto,
}

#[derive(Deserialize)]
struct SolveResponseDto {
    routes: Vec<RouteDto>,
    unserved: Vec<u32>,
    total_distance: f64,
}

#[derive(Deserialize)]
struct RouteDto {
    stops: Vec<StopDto>,
    travel_time: f64,
    wait_time: f64,
    geometry: Option<GeometryDto>,
}

#[allow(dead_code)] // reason: variants are deserialization targets
#[derive(Deserialize)]
enum StopDto {
    VehicleStart(u32),
    Customer(u32),
    Depot(u32),
    Refill(u32),
}

#[derive(Deserialize)]
struct GeometryDto {
    value: String,
}

impl StreamletRouteOptimizer {
    pub fn new(settings: &RoutingSettings) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client must build");

        let fallback;
        let effective_depots: &[_] = if settings.depots.is_empty() {
            tracing::warn!("routing.depots is empty; falling back to hardcoded Flensburg depot");
            fallback = crate::configuration::default_depots();
            &fallback
        } else {
            &settings.depots
        };

        let first = effective_depots
            .first()
            .expect("effective_depots is non-empty");
        let default_depot = LocationDto {
            lat: first.lat,
            lon: first.lon,
        };

        let refills = effective_depots
            .iter()
            .filter(|d| d.watering_point)
            .map(|d| LocationDto {
                lat: d.lat,
                lon: d.lon,
            })
            .collect();

        Self {
            client,
            solve_url: format!("{}/v1/solve", settings.streamlet_url.trim_end_matches('/')),
            default_depot,
            refills,
        }
    }

    fn build_request(
        &self,
        transporter: &Vehicle,
        trailer: Option<&Vehicle>,
        stops: &[RouteStop],
        depot: Option<Coordinate>,
    ) -> SolveRequestDto {
        let effective_depot = depot
            .map(|c| LocationDto {
                lat: c.latitude(),
                lon: c.longitude(),
            })
            .unwrap_or(self.default_depot);
        let capacity = transporter.water_capacity.liters()
            + trailer.map(|t| t.water_capacity.liters()).unwrap_or(0.0);
        let dim = transporter.dimension;
        let vehicle = VehicleDto {
            id: 1,
            start: effective_depot,
            tank: TankDto {
                capacity,
                level: capacity,
            },
            kind: KindDto::Truck {
                width: dim.width,
                height: dim.height,
                length: dim.length,
                weight: dim.weight,
            },
            shift: ShiftDto {
                start: 0.0,
                end: SHIFT_END_SECS,
            },
            max_trips: None,
        };
        let customers = stops
            .iter()
            .enumerate()
            .map(|(i, stop)| CustomerDto {
                id: (i + 1) as u32,
                location: LocationDto {
                    lat: stop.location.latitude(),
                    lon: stop.location.longitude(),
                },
                demand: stop.demand_liters,
                service_time: SERVICE_TIME_SECS,
                time_window: None,
            })
            .collect();
        let refill_stations = self
            .refills
            .iter()
            .enumerate()
            .map(|(i, location)| RefillStationDto {
                id: (i + 1) as u32,
                location: *location,
                refill_duration: REFILL_DURATION_SECS,
            })
            .collect();
        SolveRequestDto {
            problem: ProblemDto {
                vehicles: vec![vehicle],
                depots: vec![DepotDto {
                    id: 1,
                    location: effective_depot,
                }],
                customers,
                refill_stations,
            },
            options: OptionsDto {
                geometry: "polyline",
            },
        }
    }
}

// Streamlet joins multi-leg polylines with `;`.
fn decode_geometry(routes: &[RouteDto]) -> Result<Vec<Coordinate>, RoutingError> {
    let mut coords = Vec::new();
    for route in routes {
        let Some(geometry) = &route.geometry else {
            continue;
        };
        for leg in geometry.value.split(';').filter(|l| !l.is_empty()) {
            let line = polyline::decode_polyline(leg, 6)
                .map_err(|e| RoutingError::Failed(format!("invalid polyline: {e}")))?;
            for point in line.points() {
                let coordinate = Coordinate::new(point.y(), point.x())
                    .map_err(|e| RoutingError::Failed(format!("invalid route coordinate: {e}")))?;
                coords.push(coordinate);
            }
        }
    }
    Ok(coords)
}

#[async_trait]
impl RouteOptimizer for StreamletRouteOptimizer {
    #[tracing::instrument(level = "debug", skip_all, fields(stops = stops.len()))]
    async fn optimize(
        &self,
        transporter: &Vehicle,
        trailer: Option<&Vehicle>,
        stops: &[RouteStop],
        depot: Option<Coordinate>,
    ) -> Result<OptimizedRoute, RoutingError> {
        let request = self.build_request(transporter, trailer, stops, depot);
        let response = self
            .client
            .post(&self.solve_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RoutingError::Unavailable(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                422 => RoutingError::InvalidProblem(body),
                502 | 504 => RoutingError::Unavailable(body),
                _ => RoutingError::Failed(format!("streamlet returned {status}: {body}")),
            });
        }

        let solve: SolveResponseDto = response
            .json()
            .await
            .map_err(|e| RoutingError::Failed(format!("invalid streamlet response: {e}")))?;

        let distance = Distance::new(solve.total_distance.max(0.0))
            .map_err(|e| RoutingError::Failed(e.to_string()))?;
        let duration_secs: f64 = solve
            .routes
            .iter()
            .map(|r| r.travel_time + r.wait_time)
            .sum();
        let refill_count = solve
            .routes
            .iter()
            .flat_map(|r| &r.stops)
            .filter(|s| matches!(s, StopDto::Refill(_)))
            .count() as u32;
        let geometry = decode_geometry(&solve.routes)?;

        let id_by_customer: HashMap<u32, Id<TreeCluster>> = stops
            .iter()
            .enumerate()
            .map(|(i, s)| ((i + 1) as u32, s.cluster_id))
            .collect();
        let unserved = solve
            .unserved
            .iter()
            .filter_map(|id| id_by_customer.get(id).copied())
            .collect();

        Ok(OptimizedRoute {
            distance,
            duration: Duration::from_secs_f64(duration_secs.max(0.0)),
            refill_count,
            geometry,
            unserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::routing::{RouteOptimizer, RouteStop};
    use domain::shared::coordinates::Coordinate;
    use domain::vehicle::{DrivingLicense, Vehicle, VehicleSnapshot, VehicleStatus, VehicleType};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn vehicle(vehicle_type: VehicleType, water_capacity: f64) -> Vehicle {
        Vehicle::reconstitute(VehicleSnapshot {
            id: uuid::Uuid::now_v7(),
            archived_at: None,
            number_plate: "FL GE 1234".to_string(),
            description: None,
            water_capacity,
            status: VehicleStatus::Active,
            vehicle_type,
            model: "Unimog".to_string(),
            driving_license: DrivingLicense::BE,
            height: 2.1,
            width: 2.0,
            length: 5.0,
            weight: 3500.0,
            provider: None,
            additional_info: None,
        })
    }

    fn optimizer(base_url: &str) -> StreamletRouteOptimizer {
        let settings = crate::configuration::RoutingSettings {
            streamlet_url: base_url.to_string(),
            ..Default::default()
        };
        StreamletRouteOptimizer::new(&settings)
    }

    fn stops() -> Vec<RouteStop> {
        vec![
            RouteStop {
                cluster_id: domain::Id::new_v7(),
                location: Coordinate::new(54.79, 9.44).unwrap(),
                demand_liters: 160.0,
            },
            RouteStop {
                cluster_id: domain::Id::new_v7(),
                location: Coordinate::new(54.80, 9.45).unwrap(),
                demand_liters: 240.0,
            },
        ]
    }

    fn ok_body(polyline_value: &str) -> serde_json::Value {
        serde_json::json!({
            "routes": [{
                "vehicle": 1,
                "stops": [
                    {"VehicleStart": 1},
                    {"Refill": 1},
                    {"Customer": 1},
                    {"Customer": 2},
                    {"Depot": 1}
                ],
                "distance": 12500.0,
                "travel_time": 3500.0,
                "wait_time": 100.0,
                "geometry": {"format": "polyline", "value": polyline_value}
            }],
            "unserved": [],
            "total_distance": 12500.0,
            "total_travel_time": 3500.0
        })
    }

    fn encoded_line() -> String {
        // (x, y) = (lon, lat)
        let line = geo_types::LineString::from(vec![(9.4347, 54.7687), (9.4358, 54.7922)]);
        polyline::encode_coordinates(line, 6).unwrap()
    }

    #[tokio::test]
    async fn optimize_maps_request_and_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/solve"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(&encoded_line())))
            .expect(1)
            .mount(&server)
            .await;

        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        let trailer = vehicle(VehicleType::Trailer, 1000.0);
        let stops = stops();
        let route = optimizer(&server.uri())
            .optimize(&transporter, Some(&trailer), &stops, None)
            .await
            .unwrap();

        assert_eq!(route.distance.meters(), 12500.0);
        assert_eq!(route.duration.as_secs_f64(), 3600.0); // travel_time + wait_time
        assert_eq!(route.refill_count, 1);
        assert_eq!(route.geometry.len(), 2);
        assert!((route.geometry[0].latitude() - 54.7687).abs() < 1e-4);
        assert!((route.geometry[0].longitude() - 9.4347).abs() < 1e-4);
        assert!(route.unserved.is_empty());

        // request assertions
        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = requests[0].body_json().unwrap();
        // tank = transporter + trailer capacity, full level
        assert_eq!(body["problem"]["vehicles"][0]["tank"]["capacity"], 3000.0);
        assert_eq!(body["problem"]["vehicles"][0]["tank"]["level"], 3000.0);
        // truck kind carries transporter dimensions (weight in kg)
        assert_eq!(
            body["problem"]["vehicles"][0]["kind"]["Truck"]["weight"],
            3500.0
        );
        // demand passthrough, ids 1..n
        assert_eq!(body["problem"]["customers"][0]["id"], 1);
        assert_eq!(body["problem"]["customers"][0]["demand"], 160.0);
        assert_eq!(body["problem"]["customers"][1]["demand"], 240.0);
        // both depots are flagged watering_point=true → 2 refill stations
        assert_eq!(
            body["problem"]["refill_stations"].as_array().unwrap().len(),
            2
        );
        // None depot → both vehicle start and depot use the configured default (first depot)
        let default_lat = 54.76879146396569;
        let default_lon = 9.434803531218018;
        assert!(
            (body["problem"]["vehicles"][0]["start"]["lat"]
                .as_f64()
                .unwrap()
                - default_lat)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["depots"][0]["location"]["lat"]
                .as_f64()
                .unwrap()
                - default_lat)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["vehicles"][0]["start"]["lon"]
                .as_f64()
                .unwrap()
                - default_lon)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["depots"][0]["location"]["lon"]
                .as_f64()
                .unwrap()
                - default_lon)
                .abs()
                < 1e-9
        );
        // polyline geometry requested
        assert_eq!(body["options"]["geometry"], "polyline");
    }

    #[tokio::test]
    async fn optimize_uses_depot_override_for_start_and_depot() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/solve"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(&encoded_line())))
            .expect(1)
            .mount(&server)
            .await;

        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        let override_coord = Coordinate::new(54.9, 9.5).unwrap();
        optimizer(&server.uri())
            .optimize(&transporter, None, &stops(), Some(override_coord))
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = requests[0].body_json().unwrap();
        assert!(
            (body["problem"]["vehicles"][0]["start"]["lat"]
                .as_f64()
                .unwrap()
                - 54.9)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["vehicles"][0]["start"]["lon"]
                .as_f64()
                .unwrap()
                - 9.5)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["depots"][0]["location"]["lat"]
                .as_f64()
                .unwrap()
                - 54.9)
                .abs()
                < 1e-9
        );
        assert!(
            (body["problem"]["depots"][0]["location"]["lon"]
                .as_f64()
                .unwrap()
                - 9.5)
                .abs()
                < 1e-9
        );
    }

    #[tokio::test]
    async fn optimize_maps_unserved_ids_back_to_cluster_ids() {
        let server = MockServer::start().await;
        let mut body = ok_body(&encoded_line());
        body["unserved"] = serde_json::json!([2]);
        Mock::given(method("POST"))
            .and(path("/v1/solve"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        let stops = stops();
        let route = optimizer(&server.uri())
            .optimize(&transporter, None, &stops, None)
            .await
            .unwrap();
        assert_eq!(route.unserved, vec![stops[1].cluster_id]);
    }

    #[tokio::test]
    async fn optimize_maps_status_codes_to_routing_errors() {
        for (status, is_invalid, is_unavailable) in [
            (422, true, false),
            (502, false, true),
            (504, false, true),
            (500, false, false),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/v1/solve"))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
            let transporter = vehicle(VehicleType::Transporter, 2000.0);
            let err = optimizer(&server.uri())
                .optimize(&transporter, None, &stops(), None)
                .await
                .unwrap_err();
            match err {
                RoutingError::InvalidProblem(_) => assert!(is_invalid, "status {status}"),
                RoutingError::Unavailable(_) => assert!(is_unavailable, "status {status}"),
                RoutingError::Failed(_) => {
                    assert!(!is_invalid && !is_unavailable, "status {status}")
                }
            }
        }
    }

    #[tokio::test]
    async fn optimize_reports_unreachable_engine_as_unavailable() {
        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        // port 1 is never listening
        let err = optimizer("http://127.0.0.1:1")
            .optimize(&transporter, None, &stops(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, RoutingError::Unavailable(_)));
    }

    #[tokio::test]
    async fn optimize_reports_malformed_polyline_as_failed() {
        let server = MockServer::start().await;
        // "!!!" contains bytes < 63 which polyline::decode_polyline rejects
        let body = ok_body("!!!not-a-polyline");
        Mock::given(method("POST"))
            .and(path("/v1/solve"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        let err = optimizer(&server.uri())
            .optimize(&transporter, None, &stops(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, RoutingError::Failed(_)));
    }

    #[tokio::test]
    async fn unflagged_depot_is_excluded_from_refill_stations() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/solve"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_body(&encoded_line())))
            .expect(1)
            .mount(&server)
            .await;

        let settings = crate::configuration::RoutingSettings {
            streamlet_url: server.uri(),
            depots: vec![
                crate::configuration::NamedGeoPoint {
                    name: "Flagged".into(),
                    lat: 54.77,
                    lon: 9.43,
                    watering_point: true,
                },
                crate::configuration::NamedGeoPoint {
                    name: "Unflagged".into(),
                    lat: 54.80,
                    lon: 9.45,
                    watering_point: false,
                },
            ],
            ..Default::default()
        };
        let opt = StreamletRouteOptimizer::new(&settings);
        let transporter = vehicle(VehicleType::Transporter, 2000.0);
        opt.optimize(&transporter, None, &stops(), None)
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = requests[0].body_json().unwrap();
        assert_eq!(
            body["problem"]["refill_stations"].as_array().unwrap().len(),
            1
        );
        assert!(
            (body["problem"]["refill_stations"][0]["location"]["lat"]
                .as_f64()
                .unwrap()
                - 54.77)
                .abs()
                < 1e-9
        );
    }
}
