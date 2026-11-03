use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::{Method, StatusCode};
use serde_json::{Map, Value, json};
use tokio::sync::Mutex;

use domain::data_export::{
    BASE_URL_FIELD, ExportProperty, ExportTargetConfig, ExportThing, ExportUrlPolicy, RemoteId,
    SENSOR_KEY, SensorThingsSink, SinkError, SinkSession, StateStream, StateValue, TOKEN_URL_FIELD,
    location_key, property_key,
};

use super::{
    auth::{AuthFailure, Authorization, authorize},
    body::{MAX_BODY_BYTES, read_capped},
};

pub struct SensorThingsHttpSink {
    client: reqwest::Client,
    policy: ExportUrlPolicy,
}

impl SensorThingsHttpSink {
    pub fn new(client: reqwest::Client, policy: ExportUrlPolicy) -> Self {
        Self { client, policy }
    }
}

fn auth_error(failure: AuthFailure) -> SinkError {
    match failure {
        AuthFailure::Unreachable(detail) => SinkError::Unreachable(detail),
        AuthFailure::Rejected { status } => SinkError::Unauthorized(status),
        AuthFailure::Malformed(detail) => SinkError::Malformed(detail),
    }
}

#[async_trait::async_trait]
impl SensorThingsSink for SensorThingsHttpSink {
    #[tracing::instrument(level = "info", skip_all, fields(target.host = config.base_url.as_url().host_str()))]
    async fn open(&self, config: &ExportTargetConfig) -> Result<Box<dyn SinkSession>, SinkError> {
        // Every request of the session is built from `base`, so checking the two
        // configured URLs here covers all of them.
        self.policy
            .check(BASE_URL_FIELD, &config.base_url)
            .map_err(|_| SinkError::UrlNotAllowed)?;
        if let Some(token_url) = config.auth.token_url() {
            self.policy
                .check(TOKEN_URL_FIELD, token_url)
                .map_err(|_| SinkError::UrlNotAllowed)?;
        }
        let authorization = authorize(&self.client, &config.auth)
            .await
            .map_err(auth_error)?;
        Ok(Box::new(HttpSession {
            client: self.client.clone(),
            base: config.base_url.as_str().trim_end_matches('/').to_string(),
            config: config.clone(),
            authorization: Mutex::new(authorization),
        }))
    }
}

struct HttpSession {
    client: reqwest::Client,
    base: String,
    config: ExportTargetConfig,
    authorization: Mutex<Authorization>,
}

struct Answer {
    status: StatusCode,
    location: Option<String>,
    body: Vec<u8>,
}

impl HttpSession {
    async fn send(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&Value>,
    ) -> Result<Answer, SinkError> {
        for attempt in 0..2 {
            let request = {
                let authorization = self.authorization.lock().await;
                let mut request = self
                    .client
                    .request(method.clone(), format!("{}/{path}", self.base));
                if !query.is_empty() {
                    request = request.query(query);
                }
                if let Some(body) = body {
                    request = request.json(body);
                }
                authorization.apply(request)
            };
            let response = request
                .send()
                .await
                .map_err(|e| SinkError::Unreachable(e.without_url().to_string()))?;
            let status = response.status();
            if status == StatusCode::UNAUTHORIZED && attempt == 0 {
                // Tokens can expire mid-pass; one fresh token, then give up.
                *self.authorization.lock().await = authorize(&self.client, &self.config.auth)
                    .await
                    .map_err(auth_error)?;
                continue;
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            let body = read_capped(response, MAX_BODY_BYTES).await?;
            return classify(Answer {
                status,
                location,
                body,
            });
        }
        Err(SinkError::Unauthorized(401))
    }

    async fn find(&self, set: &str, ge_key: &str) -> Result<Option<RemoteId>, SinkError> {
        let filter = format!("properties/ge_key eq '{}'", ge_key.replace('\'', "''"));
        let answer = self
            .send(
                Method::GET,
                set,
                &[
                    ("$filter", filter),
                    ("$select", "id".into()),
                    ("$top", "1".into()),
                ],
                None,
            )
            .await?;
        let page: Value = parse(&answer.body)?;
        let Some(raw) = page["value"]
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item.get("@iot.id"))
        else {
            return Ok(None);
        };
        RemoteId::from_json(raw)
            .map(Some)
            .ok_or_else(|| SinkError::Malformed("unsupported id".into()))
    }

    async fn create(&self, set: &str, body: &Value) -> Result<RemoteId, SinkError> {
        let answer = self.send(Method::POST, set, &[], Some(body)).await?;
        answer
            .location
            .as_deref()
            .and_then(RemoteId::from_location)
            .ok_or_else(|| SinkError::Malformed(format!("no id in the answer to POST {set}")))
    }

    async fn find_or_create(
        &self,
        set: &str,
        ge_key: &str,
        body: &Value,
    ) -> Result<RemoteId, SinkError> {
        match self.find(set, ge_key).await? {
            Some(id) => Ok(id),
            None => self.create(set, body).await,
        }
    }
}

fn classify(answer: Answer) -> Result<Answer, SinkError> {
    let status = answer.status;
    if status.is_success() {
        return Ok(answer);
    }
    let code = status.as_u16();
    if matches!(code, 401 | 403) {
        return Err(SinkError::Unauthorized(code));
    }
    if status.is_client_error() {
        let detail: String = String::from_utf8_lossy(&answer.body)
            .chars()
            .take(300)
            .collect();
        return Err(SinkError::Rejected {
            status: code,
            detail,
        });
    }
    Err(SinkError::Unreachable(format!("HTTP {code}")))
}

fn parse(body: &[u8]) -> Result<Value, SinkError> {
    serde_json::from_slice(body).map_err(|e| SinkError::Malformed(e.to_string()))
}

fn rfc3339(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn thing_properties(thing: &ExportThing) -> Value {
    let mut properties: Map<String, Value> = thing.attributes.clone().into_iter().collect();
    properties.insert("ge_key".into(), json!(thing.key()));
    properties.insert("source".into(), json!("green-ecolution"));
    properties.insert("module".into(), json!(thing.target.module.as_str()));
    properties.insert("kind".into(), json!(thing.target.kind.as_str()));
    properties.insert("id".into(), json!(thing.target.id.to_string()));
    Value::Object(properties)
}

fn location_body(thing: &ExportThing) -> Option<Value> {
    thing.coordinate.map(|c| {
        json!({
            "name": thing.name,
            "description": "Standort",
            "encodingType": "application/geo+json",
            "location": { "type": "Point", "coordinates": [c.longitude(), c.latitude()] },
            "properties": { "ge_key": location_key(&thing.key()) },
        })
    })
}

fn id_link(id: &RemoteId) -> Value {
    json!({ "@iot.id": id.as_json() })
}

#[async_trait::async_trait]
impl SinkSession for HttpSession {
    async fn create_thing(&self, thing: &ExportThing) -> Result<(RemoteId, bool), SinkError> {
        if let Some(id) = self.find("Things", &thing.key()).await? {
            self.update_thing(&id, thing).await?;
            return Ok((id, false));
        }
        let mut body = json!({
            "name": thing.name,
            "description": thing.description,
            "properties": thing_properties(thing),
        });
        if let Some(location) = location_body(thing) {
            body["Locations"] = json!([location]);
        }
        Ok((self.create("Things", &body).await?, true))
    }

    async fn update_thing(&self, id: &RemoteId, thing: &ExportThing) -> Result<(), SinkError> {
        let body = json!({
            "name": thing.name,
            "description": thing.description,
            "properties": thing_properties(thing),
        });
        self.send(
            Method::PATCH,
            &format!("Things{}", id.path_segment()),
            &[],
            Some(&body),
        )
        .await?;
        Ok(())
    }

    async fn move_thing(&self, id: &RemoteId, thing: &ExportThing) -> Result<(), SinkError> {
        let Some(mut location) = location_body(thing) else {
            return Ok(());
        };
        // A new Location linked to the Thing replaces its current one; FROST keeps the history.
        location["Things"] = json!([id_link(id)]);
        self.create("Locations", &location).await?;
        Ok(())
    }

    async fn mark_thing_deleted(
        &self,
        id: &RemoteId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), SinkError> {
        let path = format!("Things{}", id.path_segment());
        let answer = self
            .send(
                Method::GET,
                &path,
                &[("$select", "properties".into())],
                None,
            )
            .await?;
        let current: Value = parse(&answer.body)?;
        let mut properties = current["properties"]
            .as_object()
            .cloned()
            .ok_or_else(|| SinkError::Malformed("thing has no properties object".into()))?;
        properties.insert("deleted_at".into(), json!(rfc3339(deleted_at)));
        self.send(
            Method::PATCH,
            &path,
            &[],
            Some(&json!({ "properties": properties })),
        )
        .await?;
        Ok(())
    }

    async fn ensure_sensor(&self) -> Result<RemoteId, SinkError> {
        let body = json!({
            "name": "Green Ecolution",
            "description": "Derived states computed by Green Ecolution",
            "encodingType": "text/plain",
            "metadata": "Green Ecolution",
            "properties": { "ge_key": SENSOR_KEY },
        });
        self.find_or_create("Sensors", SENSOR_KEY, &body).await
    }

    async fn ensure_observed_property(
        &self,
        property: &ExportProperty,
    ) -> Result<RemoteId, SinkError> {
        let key = property_key(property);
        let body = json!({
            "name": property.label,
            "description": property.description,
            "definition": format!("urn:green-ecolution:property:{}", property.key),
            "properties": { "ge_key": key },
        });
        self.find_or_create("ObservedProperties", &key, &body).await
    }

    async fn ensure_state_datastream(
        &self,
        stream: &StateStream<'_>,
    ) -> Result<RemoteId, SinkError> {
        let unit = match stream.property.unit {
            Some(u) => json!({ "name": u.name, "symbol": u.symbol, "definition": u.definition }),
            None => json!({ "name": null, "symbol": null, "definition": null }),
        };
        let body = json!({
            "name": format!("{} · {}", stream.thing_name, stream.property.label),
            "description": stream.property.description,
            "observationType": stream.property.kind.observation_type(),
            "unitOfMeasurement": unit,
            "Thing": id_link(stream.thing),
            "Sensor": id_link(stream.sensor),
            "ObservedProperty": id_link(stream.observed_property),
            "properties": { "ge_key": stream.ge_key },
        });
        self.find_or_create("Datastreams", stream.ge_key, &body)
            .await
    }

    async fn post_observation(
        &self,
        datastream: &RemoteId,
        value: &StateValue,
        phenomenon_time: DateTime<Utc>,
    ) -> Result<(), SinkError> {
        let body = json!({ "phenomenonTime": rfc3339(phenomenon_time), "result": value.to_json() });
        self.create(
            &format!("Datastreams{}/Observations", datastream.path_segment()),
            &body,
        )
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::sensorthings::http_client;
    use chrono::TimeZone;
    use domain::{
        data_export::{
            BASE_URL_FIELD, ExportAuth, ExportTargetConfig, ExportThing, ExportUrl,
            ExportUrlPolicy, FILL_LEVEL, StateValue, WATERING_STATUS,
        },
        shared::coordinates::Coordinate,
        target::TargetRef,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use uuid::Uuid;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_partial_json, method, path, query_param},
    };

    fn policy() -> ExportUrlPolicy {
        ExportUrlPolicy::new(["127.0.0.1".to_string()], true)
    }

    fn config(server: &MockServer) -> ExportTargetConfig {
        ExportTargetConfig {
            enabled: true,
            base_url: ExportUrl::service_root(BASE_URL_FIELD, &format!("{}/v1.1", server.uri()))
                .unwrap(),
            auth: ExportAuth::None,
        }
    }

    fn tree() -> ExportThing {
        ExportThing {
            target: TargetRef::new("irrigation", "tree", Uuid::from_u128(1)).unwrap(),
            name: "Baum 12".into(),
            description: "Linde".into(),
            attributes: BTreeMap::from([("species".into(), json!("Linde"))]),
            coordinate: Some(Coordinate::new(54.79, 9.43).unwrap()),
            states: vec![],
        }
    }

    fn created(id: i64, set: &str, server: &MockServer) -> ResponseTemplate {
        ResponseTemplate::new(201).insert_header(
            "location",
            format!("{}/v1.1/{set}({id})", server.uri()).as_str(),
        )
    }

    fn none_found() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({ "value": [] }))
    }

    async fn session(server: &MockServer) -> Box<dyn SinkSession> {
        SensorThingsHttpSink::new(http_client(), policy())
            .open(&config(server))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_new_thing_is_created_with_its_location_and_identifying_properties() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Things"))
            .and(query_param(
                "$filter",
                "properties/ge_key eq 'irrigation:tree:00000000-0000-0000-0000-000000000001'",
            ))
            .respond_with(none_found())
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Things"))
            .and(body_partial_json(json!({
                "name": "Baum 12",
                "properties": {
                    "species": "Linde",
                    "ge_key": "irrigation:tree:00000000-0000-0000-0000-000000000001",
                    "source": "green-ecolution", "module": "irrigation", "kind": "tree",
                    "id": "00000000-0000-0000-0000-000000000001"
                },
                "Locations": [{ "location": { "type": "Point", "coordinates": [9.43, 54.79] } }]
            })))
            .respond_with(created(5, "Things", &server))
            .expect(1)
            .mount(&server)
            .await;

        let (id, new) = session(&server).await.create_thing(&tree()).await.unwrap();

        assert_eq!(id, RemoteId(json!(5)));
        assert!(new);
    }

    #[tokio::test]
    async fn an_existing_thing_with_the_same_key_is_patched_instead_of_duplicated() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Things"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({ "value": [{ "@iot.id": 9 }] })),
            )
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .and(path("/v1.1/Things(9)"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .mount(&server)
            .await;

        let (id, new) = session(&server).await.create_thing(&tree()).await.unwrap();

        assert_eq!(id, RemoteId(json!(9)));
        assert!(!new);
    }

    #[tokio::test]
    async fn a_move_posts_a_new_location_linked_to_the_thing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Locations"))
            .and(body_partial_json(json!({ "Things": [{ "@iot.id": 5 }] })))
            .respond_with(created(3, "Locations", &server))
            .expect(1)
            .mount(&server)
            .await;

        session(&server)
            .await
            .move_thing(&RemoteId(json!(5)), &tree())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn marking_deleted_keeps_the_other_properties() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Things(5)"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "properties": { "species": "Linde" } })),
            )
            .mount(&server)
            .await;
        Mock::given(method("PATCH")).and(path("/v1.1/Things(5)"))
            .and(body_partial_json(json!({ "properties": { "species": "Linde", "deleted_at": "2026-11-03T12:00:00Z" } })))
            .respond_with(ResponseTemplate::new(200)).expect(1).mount(&server).await;

        let at = chrono::Utc.with_ymd_and_hms(2026, 11, 3, 12, 0, 0).unwrap();
        session(&server)
            .await
            .mark_thing_deleted(&RemoteId(json!(5)), at)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn an_id_that_is_neither_number_nor_string_is_refused() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Things"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({ "value": [{ "@iot.id": { "x": 1 } }] })),
            )
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(201))
            .expect(0)
            .mount(&server)
            .await;

        let result = session(&server).await.create_thing(&tree()).await;

        assert!(matches!(result, Err(SinkError::Malformed(ref m)) if m == "unsupported id"));
    }

    #[tokio::test]
    async fn marking_deleted_without_properties_sends_nothing() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Things(5)"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "name": "x" })))
            .mount(&server)
            .await;
        Mock::given(method("PATCH"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;

        let result = session(&server)
            .await
            .mark_thing_deleted(&RemoteId(json!(5)), chrono::Utc::now())
            .await;

        assert!(matches!(result, Err(SinkError::Malformed(_))));
    }

    #[tokio::test]
    async fn a_401_is_retried_once_with_fresh_authorization() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(8)/Observations"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(8)/Observations"))
            .respond_with(created(1, "Observations", &server))
            .mount(&server)
            .await;

        session(&server)
            .await
            .post_observation(
                &RemoteId(json!(8)),
                &StateValue::Measurement(1.0),
                chrono::Utc::now(),
            )
            .await
            .unwrap();

        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn a_second_401_is_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(8)/Observations"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let result = session(&server)
            .await
            .post_observation(
                &RemoteId(json!(8)),
                &StateValue::Measurement(1.0),
                chrono::Utc::now(),
            )
            .await;

        assert!(matches!(result, Err(SinkError::Unauthorized(401))));
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn a_category_datastream_has_a_null_unit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Datastreams"))
            .respond_with(none_found())
            .mount(&server)
            .await;
        Mock::given(method("POST")).and(path("/v1.1/Datastreams"))
            .and(body_partial_json(json!({
                "name": "Baum 12 · Bewässerungsstatus",
                "unitOfMeasurement": { "name": null, "symbol": null, "definition": null },
                "Thing": { "@iot.id": 5 }, "Sensor": { "@iot.id": 1 }, "ObservedProperty": { "@iot.id": 2 }
            })))
            .respond_with(created(8, "Datastreams", &server)).expect(1).mount(&server).await;

        let id = session(&server)
            .await
            .ensure_state_datastream(&StateStream {
                ge_key: "state:watering_status:irrigation:tree:1",
                thing: &RemoteId(json!(5)),
                thing_name: "Baum 12",
                property: &WATERING_STATUS,
                sensor: &RemoteId(json!(1)),
                observed_property: &RemoteId(json!(2)),
            })
            .await
            .unwrap();
        assert_eq!(id, RemoteId(json!(8)));
    }

    #[tokio::test]
    async fn a_measurement_datastream_carries_its_unit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Datastreams"))
            .respond_with(none_found())
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams"))
            .and(body_partial_json(
                json!({ "unitOfMeasurement": { "symbol": "%" } }),
            ))
            .respond_with(created(8, "Datastreams", &server))
            .expect(1)
            .mount(&server)
            .await;

        session(&server)
            .await
            .ensure_state_datastream(&StateStream {
                ge_key: "state:fill_level:waste:bin:1",
                thing: &RemoteId(json!(5)),
                thing_name: "Behälter 3",
                property: &FILL_LEVEL,
                sensor: &RemoteId(json!(1)),
                observed_property: &RemoteId(json!(2)),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn observations_are_posted_to_their_datastream() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(8)/Observations"))
            .and(body_partial_json(
                json!({ "result": "good", "phenomenonTime": "2026-11-03T12:00:00Z" }),
            ))
            .respond_with(created(1, "Observations", &server))
            .expect(1)
            .mount(&server)
            .await;

        let at = chrono::Utc.with_ymd_and_hms(2026, 11, 3, 12, 0, 0).unwrap();
        session(&server)
            .await
            .post_observation(
                &RemoteId(json!(8)),
                &StateValue::Category("good".into()),
                at,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn status_codes_map_to_sink_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(1)/Observations"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad result"))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(2)/Observations"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1.1/Datastreams(3)/Observations"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&server)
            .await;
        let s = session(&server).await;
        let at = chrono::Utc::now();
        let v = StateValue::Measurement(1.0);

        assert!(
            matches!(s.post_observation(&RemoteId(json!(1)), &v, at).await,
            Err(SinkError::Rejected { status: 400, ref detail }) if detail == "bad result")
        );
        assert!(matches!(
            s.post_observation(&RemoteId(json!(2)), &v, at).await,
            Err(SinkError::Unreachable(_))
        ));
        assert!(matches!(
            s.post_observation(&RemoteId(json!(3)), &v, at).await,
            Err(SinkError::Unauthorized(403))
        ));
    }

    #[tokio::test]
    async fn an_oversized_answer_is_refused() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1.1/Sensors"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("x".repeat(MAX_BODY_BYTES + 1)),
            )
            .mount(&server)
            .await;

        assert!(matches!(
            session(&server).await.ensure_sensor().await,
            Err(SinkError::Malformed(_))
        ));
    }

    #[tokio::test]
    async fn opening_a_session_for_a_host_off_the_list_sends_nothing() {
        let server = MockServer::start().await;
        let result = SensorThingsHttpSink::new(
            http_client(),
            ExportUrlPolicy::new(Vec::<String>::new(), true),
        )
        .open(&config(&server))
        .await;

        assert!(matches!(result, Err(SinkError::UrlNotAllowed)));
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}
