use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};

use domain::routing::{RoutingError, TurnByTurnRequest, TurnByTurnRoute, TurnByTurnRouter};

use crate::configuration::RoutingSettings;

const TIMEOUT: Duration = Duration::from_secs(15);
const HEADING_TOLERANCE_DEG: u32 = 45;

pub struct ValhallaTurnByTurnRouter {
    client: reqwest::Client,
    route_url: String,
}

impl ValhallaTurnByTurnRouter {
    pub fn new(settings: &RoutingSettings) -> Self {
        Self::with_url(&settings.valhalla_url)
    }

    pub fn with_url(base: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("reqwest client with only a timeout always builds"),
            route_url: format!("{}/route", base.trim_end_matches('/')),
        }
    }
}

/// Ferrostar's Valhalla adapter parses OSRM-format answers with banner and voice
/// instructions, so the request asks for exactly that.
pub fn request_body(req: &TurnByTurnRequest) -> Value {
    let mut origin = json!({
        "lat": req.origin.latitude(),
        "lon": req.origin.longitude(),
        "type": "break",
    });
    if let Some(heading) = req.heading {
        origin["heading"] = json!(heading);
        origin["heading_tolerance"] = json!(HEADING_TOLERANCE_DEG);
    }
    let mut locations = vec![origin];
    locations.extend(
        req.waypoints
            .iter()
            .map(|c| json!({ "lat": c.latitude(), "lon": c.longitude(), "type": "break" })),
    );
    json!({
        "locations": locations,
        "costing": "truck",
        "costing_options": { "truck": {
            "height": req.vehicle.height,
            "width": req.vehicle.width,
            "length": req.vehicle.length,
            // Valhalla expects metric tons; vehicles store kilograms.
            "weight": req.vehicle.weight / 1000.0,
        }},
        "units": "kilometers",
        "language": "de-DE",
        "directions_type": "instructions",
        "format": "osrm",
        "banner_instructions": true,
        "voice_instructions": true,
    })
}

#[async_trait]
impl TurnByTurnRouter for ValhallaTurnByTurnRouter {
    #[tracing::instrument(level = "debug", skip_all, fields(waypoints = request.waypoints.len()))]
    async fn route(&self, request: TurnByTurnRequest) -> Result<TurnByTurnRoute, RoutingError> {
        let response = self
            .client
            .post(&self.route_url)
            .json(&request_body(&request))
            .send()
            .await
            .map_err(|e| RoutingError::Unavailable(e.to_string()))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| RoutingError::Unavailable(e.to_string()))?;
        if status.is_success() {
            Ok(TurnByTurnRoute { json: text })
        } else if status.is_client_error() {
            Err(RoutingError::InvalidProblem(text))
        } else {
            Err(RoutingError::Unavailable(format!(
                "valhalla answered {status}"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::shared::coordinates::Coordinate;
    use domain::vehicle::VehicleDimension;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn request() -> TurnByTurnRequest {
        TurnByTurnRequest {
            origin: Coordinate::new(54.79, 9.43).unwrap(),
            heading: Some(87.0),
            waypoints: vec![Coordinate::new(54.80, 9.44).unwrap()],
            vehicle: VehicleDimension::new(3.2, 2.5, 8.0, 12000.0).unwrap(),
        }
    }

    #[test]
    fn body_asks_for_osrm_format_with_banners_and_voice_in_german() {
        let body = request_body(&request());
        assert_eq!(body["format"], "osrm");
        assert_eq!(body["banner_instructions"], true);
        assert_eq!(body["voice_instructions"], true);
        assert_eq!(body["language"], "de-DE");
        assert_eq!(body["costing"], "truck");
    }

    #[test]
    fn origin_carries_heading_and_weight_goes_in_tons() {
        let body = request_body(&request());
        assert_eq!(body["locations"][0]["heading"], 87.0);
        assert_eq!(body["locations"].as_array().unwrap().len(), 2);
        assert_eq!(body["costing_options"]["truck"]["weight"], 12.0);
        assert_eq!(body["costing_options"]["truck"]["height"], 3.2);
    }

    #[tokio::test]
    async fn passes_the_answer_through() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/route"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"routes":[]}"#))
            .mount(&server)
            .await;
        let router = ValhallaTurnByTurnRouter::with_url(&server.uri());
        let route = router.route(request()).await.unwrap();
        assert_eq!(route.json, r#"{"routes":[]}"#);
    }

    #[tokio::test]
    async fn a_400_is_an_invalid_problem_and_a_500_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/route"))
            .respond_with(ResponseTemplate::new(400).set_body_string(r#"{"error":"No path"}"#))
            .mount(&server)
            .await;
        let err = ValhallaTurnByTurnRouter::with_url(&server.uri())
            .route(request())
            .await
            .unwrap_err();
        assert!(matches!(err, RoutingError::InvalidProblem(_)));

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/route"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let err = ValhallaTurnByTurnRouter::with_url(&server.uri())
            .route(request())
            .await
            .unwrap_err();
        assert!(matches!(err, RoutingError::Unavailable(_)));
    }
}
