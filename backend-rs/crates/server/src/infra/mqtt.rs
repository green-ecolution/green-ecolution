//! MQTT ingestor: subscribes to a broker topic, looks up the sensor model
//! to pick the right wire-format parser, and forwards the normalized reading
//! to [`SensorService::ingest_reading`].
//!
//! Disabled by default; flip `mqtt.enabled = true` (and set `broker_url` /
//! `topic`) to start the background task. The task reconnects automatically
//! through `rumqttc`'s `EventLoop` and survives broker outages.

use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS, Transport};
use rust_decimal::Decimal;
use secrecy::ExposeSecret;
use serde_json::Value;
use url::Url;

use crate::{
    configuration::MqttSettings,
    service::sensor_service::{ReadingIngest, SensorService},
};
use domain::{
    events::SensorReadings,
    sensor::{
        SensorId,
        data::{VolumetricReading, Watermark},
        payload::{EcoDrizzlerPayload, GenericReadingPayload, PayloadReading},
        repository::NormalizedValue,
    },
    sensor_model::{SensorAbilityName, SensorModel},
};

#[derive(Default)]
pub struct MqttHealthState {
    pub connected: AtomicBool,
}

/// Spawns the MQTT subscriber as a tokio task. Returns `Ok(Arc<MqttHealthState>)` if
/// disabled or if the task started successfully. The task itself logs and recovers
/// from connection errors; the caller does not await its completion.
pub fn spawn(
    settings: MqttSettings,
    sensor_service: Arc<SensorService>,
) -> Result<Arc<MqttHealthState>, MqttSubscriberError> {
    let state = Arc::new(MqttHealthState::default());

    if !settings.enabled {
        tracing::info!("mqtt subscriber disabled due to config (mqtt.enabled = false)");
        return Ok(state);
    }
    if settings.broker_url.is_empty() || settings.topic.is_empty() {
        return Err(MqttSubscriberError::MissingConfig);
    }

    let (client, eventloop) = build_client(&settings)?;
    let topic = settings.topic.clone();
    let task_state = state.clone();

    tokio::spawn(async move {
        if let Err(e) = client.subscribe(&topic, QoS::AtLeastOnce).await {
            tracing::error!(error = %e, mqtt.topic = %topic, "mqtt subscribe failed");
            return;
        }
        run_event_loop(eventloop, sensor_service, task_state).await;
    });

    Ok(state)
}

fn build_client(settings: &MqttSettings) -> Result<(AsyncClient, EventLoop), MqttSubscriberError> {
    let url = Url::parse(&settings.broker_url).map_err(|e| {
        MqttSubscriberError::InvalidBrokerUrl(format!("{}: {e}", settings.broker_url))
    })?;
    let host = url
        .host_str()
        .ok_or_else(|| MqttSubscriberError::InvalidBrokerUrl("missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(match url.scheme() {
        "mqtts" | "ssl" => 8883,
        _ => 1883,
    });

    let mut opts = MqttOptions::new(&settings.client_id, host, port);
    opts.set_keep_alive(Duration::from_secs(settings.keep_alive_secs as u64));
    if matches!(url.scheme(), "mqtts" | "ssl") {
        opts.set_transport(Transport::tls_with_default_config());
    }
    if let (Some(user), Some(pwd)) = (&settings.username, &settings.password) {
        opts.set_credentials(user, pwd.expose_secret());
    }

    Ok(AsyncClient::new(opts, 32))
}

async fn run_event_loop(
    mut eventloop: EventLoop,
    sensor_service: Arc<SensorService>,
    state: Arc<MqttHealthState>,
) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                state.connected.store(true, Ordering::Relaxed);
            }
            Ok(Event::Incoming(Incoming::Publish(pub_pkt))) => {
                if let Err(e) = handle_publish(&pub_pkt.payload, &sensor_service).await {
                    tracing::warn!(error = %e, "mqtt message dropped");
                }
            }
            Ok(Event::Incoming(Incoming::Disconnect)) => {
                state.connected.store(false, Ordering::Relaxed);
            }
            Ok(_) => {}
            Err(e) => {
                state.connected.store(false, Ordering::Relaxed);
                tracing::warn!(error = %e, "mqtt eventloop error; reconnecting");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn handle_publish(
    raw: &[u8],
    sensor_service: &SensorService,
) -> Result<(), MqttSubscriberError> {
    let envelope: Value =
        serde_json::from_slice(raw).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;
    let device = pick_device(&envelope)?;
    let body = unwrap_envelope(&envelope);
    let sensor_id = SensorId::new(device)
        .map_err(|e| MqttSubscriberError::Decode(format!("invalid sensor id: {e}")))?;

    let sensor = sensor_service
        .by_id(&sensor_id)
        .await
        .map_err(|e| MqttSubscriberError::Service(e.to_string()))?;
    let model = sensor_service
        .model_by_id(sensor.model_id())
        .await
        .map_err(|e| MqttSubscriberError::Service(e.to_string()))?;

    let ingest = build_ingest(&model, &body, sensor_id)?;
    sensor_service
        .ingest_reading(ingest)
        .await
        .map_err(|e| MqttSubscriberError::Service(e.to_string()))
}

/// Handles both TTN envelopes (`data.uplink_message.decoded_payload` and
/// `uplink_message.decoded_payload`) and a flat body for direct publish tests.
fn unwrap_envelope(v: &Value) -> Value {
    if let Some(decoded) = v
        .pointer("/data/uplink_message/decoded_payload")
        .or_else(|| v.pointer("/uplink_message/decoded_payload"))
    {
        return decoded.clone();
    }
    v.clone()
}

/// `identifiers[]` is scanned, not indexed: TTN does not guarantee that
/// `device_ids` is the first entry.
fn pick_device(v: &Value) -> Result<String, MqttSubscriberError> {
    let direct = [
        "/data/end_device_ids/device_id",
        "/end_device_ids/device_id",
        "/device",
        "/deviceName",
    ];
    for p in direct {
        if let Some(s) = v.pointer(p).and_then(|x| x.as_str()) {
            return Ok(s.to_owned());
        }
    }
    if let Some(arr) = v.pointer("/identifiers").and_then(|x| x.as_array()) {
        for entry in arr {
            if let Some(s) = entry
                .pointer("/device_ids/device_id")
                .and_then(|x| x.as_str())
            {
                return Ok(s.to_owned());
            }
        }
    }
    Err(MqttSubscriberError::Decode("missing device id".into()))
}

fn build_ingest(
    model: &SensorModel,
    body: &Value,
    sensor_id: SensorId,
) -> Result<ReadingIngest, MqttSubscriberError> {
    match model.name.as_str() {
        "EcoDrizzler" => build_eco_drizzler(model, body, sensor_id),
        "GES-1000" => build_ges_1000(model, body, sensor_id),
        other => Err(MqttSubscriberError::Decode(format!(
            "no parser registered for sensor model {other}"
        ))),
    }
}

fn build_eco_drizzler(
    model: &SensorModel,
    body: &Value,
    sensor_id: SensorId,
) -> Result<ReadingIngest, MqttSubscriberError> {
    let payload: EcoDrizzlerPayload = match serde_json::from_value(body.clone()) {
        Ok(p) => p,
        Err(_) => decode_ttn_eco_drizzler(body)?,
    };
    let raw_payload =
        serde_json::to_value(&payload).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;
    let normalized = normalize_eco_drizzler(model, &payload);
    Ok(ReadingIngest {
        sensor_id,
        raw_payload,
        normalized,
        typed: SensorReadings::Watermarks(payload.watermarks),
    })
}

fn build_ges_1000(
    model: &SensorModel,
    body: &Value,
    sensor_id: SensorId,
) -> Result<ReadingIngest, MqttSubscriberError> {
    let payload: GenericReadingPayload = match serde_json::from_value(body.clone()) {
        Ok(p) => p,
        Err(e) => {
            tracing::trace!(
                error = %e,
                "generic ges-1000 payload decode failed; falling back to TTN shape"
            );
            decode_ttn_ges_1000(body, &sensor_id)?
        }
    };
    let raw_payload =
        serde_json::to_value(&payload).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;

    let mut normalized = Vec::with_capacity(payload.readings.len());
    let mut volumetrics = Vec::new();
    for r in &payload.readings {
        let name = match SensorAbilityName::from_str(&r.ability) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(error = %e, sensor.ability = %r.ability, "dropping reading with unknown ability");
                continue;
            }
        };
        let Some(model_ability_id) = model.ability_id_for(name, r.depth) else {
            tracing::warn!(
                sensor.ability = %r.ability,
                sensor.depth_cm = r.depth,
                "dropping reading with no matching model ability"
            );
            continue;
        };
        normalized.push(NormalizedValue {
            model_ability_id,
            value: Decimal::from_f64_retain(r.value).unwrap_or_default(),
        });
        if name == SensorAbilityName::SoilMoisture {
            volumetrics.push(VolumetricReading {
                depth_cm: r.depth,
                moisture_percent: r.value,
            });
        }
    }

    Ok(ReadingIngest {
        sensor_id,
        raw_payload,
        normalized,
        typed: SensorReadings::Volumetrics(volumetrics),
    })
}

fn normalize_eco_drizzler(
    model: &SensorModel,
    payload: &EcoDrizzlerPayload,
) -> Vec<NormalizedValue> {
    let mut out = Vec::with_capacity(payload.watermarks.len() + 2);
    for w in &payload.watermarks {
        if let Some(model_ability_id) =
            model.ability_id_for(SensorAbilityName::SoilTension, w.depth)
        {
            out.push(NormalizedValue {
                model_ability_id,
                value: Decimal::from(w.centibar),
            });
        }
    }
    if let Some(model_ability_id) = model.ability_id_for(SensorAbilityName::Temperature, 15) {
        out.push(NormalizedValue {
            model_ability_id,
            value: Decimal::from_f64_retain(payload.temperature).unwrap_or_default(),
        });
    }
    if let Some(model_ability_id) = model.ability_id_for(SensorAbilityName::Humidity, 15) {
        out.push(NormalizedValue {
            model_ability_id,
            value: Decimal::from_f64_retain(payload.humidity).unwrap_or_default(),
        });
    }
    out
}

/// Translates the legacy TTN-decoded EcoDrizzler shape (Resistance/CB pairs)
/// into the flat [`EcoDrizzlerPayload`] schema. Kept here so production
/// deployments that still emit the old key-per-depth layout keep working.
fn decode_ttn_eco_drizzler(decoded: &Value) -> Result<EcoDrizzlerPayload, MqttSubscriberError> {
    let device = pick_string(decoded, "deviceName")?;
    let battery = pick_f64(decoded, "batteryVoltage")?;
    let humidity = pick_f64(decoded, "waterContent")?;
    let temperature = pick_f64(decoded, "temperature")?;
    let latitude = pick_f64(decoded, "latitude")?;
    let longitude = pick_f64(decoded, "longitude")?;
    let watermarks = vec![
        watermark_at(decoded, 30, "WM30_Resistance", "WM30_CB")?,
        watermark_at(decoded, 60, "WM60_Resistance", "WM60_CB")?,
        watermark_at(decoded, 90, "WM90_Resistance", "WM90_CB")?,
    ];
    Ok(EcoDrizzlerPayload {
        device,
        battery,
        humidity,
        temperature,
        latitude,
        longitude,
        watermarks,
    })
}

/// Depth comes from the probe `id`, not the JSON key position, so swapped
/// probes or reordered keys do not silently invert depths. Battery-only
/// payloads are rejected to avoid emitting an empty `Volumetrics` event.
fn decode_ttn_ges_1000(
    decoded: &Value,
    sensor_id: &SensorId,
) -> Result<GenericReadingPayload, MqttSubscriberError> {
    let battery = decoded.get("battery_v").and_then(|x| x.as_f64());
    let mut readings = Vec::new();
    for key in ["sensor1", "sensor2"] {
        let Some(s) = decoded.get(key) else { continue };
        let probe_id = s.get("id").and_then(|x| x.as_i64());
        let Some(depth) = probe_id.and_then(probe_id_to_depth_cm) else {
            tracing::warn!(
                sensor.probe = %key,
                sensor.probe_id = ?probe_id,
                "ges-1000 probe has missing or unknown id; skipping readings"
            );
            continue;
        };
        if let Some(v) = s.get("humidity").and_then(|x| x.as_f64()) {
            readings.push(PayloadReading {
                ability: "soil_moisture".into(),
                depth,
                value: v,
            });
        }
        if let Some(v) = s.get("temperature").and_then(|x| x.as_f64()) {
            readings.push(PayloadReading {
                ability: "temperature".into(),
                depth,
                value: v,
            });
        }
    }
    if readings.is_empty() {
        return Err(MqttSubscriberError::Decode(
            "ges-1000 payload has no probe readings (sensor1/sensor2 missing or empty)".into(),
        ));
    }
    if let Some(v) = battery {
        readings.push(PayloadReading {
            ability: "battery".into(),
            depth: 0,
            value: v,
        });
    }
    Ok(GenericReadingPayload {
        device: sensor_id.as_str().to_owned(),
        battery,
        readings,
    })
}

/// GES-1000 firmware assigns id 1 to the 40 cm probe, id 2 to 80 cm.
fn probe_id_to_depth_cm(id: i64) -> Option<i32> {
    match id {
        1 => Some(40),
        2 => Some(80),
        _ => None,
    }
}

fn pick_string(v: &Value, key: &str) -> Result<String, MqttSubscriberError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| MqttSubscriberError::Decode(format!("missing string field {key}")))
}

fn pick_f64(v: &Value, key: &str) -> Result<f64, MqttSubscriberError> {
    v.get(key)
        .and_then(|x| x.as_f64())
        .ok_or_else(|| MqttSubscriberError::Decode(format!("missing numeric field {key}")))
}

fn watermark_at(
    v: &Value,
    depth: i32,
    res_key: &str,
    cb_key: &str,
) -> Result<Watermark, MqttSubscriberError> {
    Ok(Watermark {
        depth,
        resistance: pick_f64(v, res_key)? as i32,
        centibar: pick_f64(v, cb_key)? as i32,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum MqttSubscriberError {
    #[error("mqtt config missing required fields (broker_url, topic)")]
    MissingConfig,
    #[error("invalid broker url: {0}")]
    InvalidBrokerUrl(String),
    #[error("mqtt payload decode failed: {0}")]
    Decode(String),
    #[error("sensor service rejected message: {0}")]
    Service(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        Id,
        sensor_model::{SensorAbility, SensorAbilityUnit, SensorModelAbility, SensorModelName},
    };
    use serde_json::json;

    fn eco_drizzler_model() -> SensorModel {
        let tension_ability_id = uuid::Uuid::now_v7();
        let st = move |depth_cm| SensorModelAbility {
            id: uuid::Uuid::now_v7(),
            ability: SensorAbility {
                id: tension_ability_id,
                name: SensorAbilityName::SoilTension,
                unit: SensorAbilityUnit::Centibar,
            },
            depth_cm,
        };
        SensorModel {
            id: Id::new_v7(),
            name: SensorModelName::new("EcoDrizzler").unwrap(),
            description: None,
            abilities: vec![st(30), st(60), st(90)],
        }
    }

    /// Mirrors the production schema set by migration 20260527124126.
    fn ges_1000_model() -> SensorModel {
        fn ma(
            ability_id: uuid::Uuid,
            name: SensorAbilityName,
            unit: SensorAbilityUnit,
            depth_cm: i32,
        ) -> SensorModelAbility {
            SensorModelAbility {
                id: uuid::Uuid::now_v7(),
                ability: SensorAbility {
                    id: ability_id,
                    name,
                    unit,
                },
                depth_cm,
            }
        }
        let moisture_id = uuid::Uuid::now_v7();
        let temperature_id = uuid::Uuid::now_v7();
        let battery_id = uuid::Uuid::now_v7();
        SensorModel {
            id: Id::new_v7(),
            name: SensorModelName::new("GES-1000").unwrap(),
            description: None,
            abilities: vec![
                ma(
                    moisture_id,
                    SensorAbilityName::SoilMoisture,
                    SensorAbilityUnit::Percent,
                    40,
                ),
                ma(
                    moisture_id,
                    SensorAbilityName::SoilMoisture,
                    SensorAbilityUnit::Percent,
                    80,
                ),
                ma(
                    temperature_id,
                    SensorAbilityName::Temperature,
                    SensorAbilityUnit::Celsius,
                    40,
                ),
                ma(
                    temperature_id,
                    SensorAbilityName::Temperature,
                    SensorAbilityUnit::Celsius,
                    80,
                ),
                ma(
                    battery_id,
                    SensorAbilityName::Battery,
                    SensorAbilityUnit::Volt,
                    0,
                ),
            ],
        }
    }

    #[test]
    fn unwrap_envelope_handles_ttn_and_flat() {
        let env = json!({"uplink_message": {"decoded_payload": {"device": "eui-1"}}});
        let v = unwrap_envelope(&env);
        assert_eq!(v["device"], "eui-1");

        let flat = json!({"device": "eui-2"});
        let v = unwrap_envelope(&flat);
        assert_eq!(v["device"], "eui-2");
    }

    #[test]
    fn unwrap_envelope_handles_as_up_data_forward() {
        let env = json!({
            "name": "as.up.data.forward",
            "data": {
                "end_device_ids": {"device_id": "eui-a8404107bf5e6409"},
                "uplink_message": {
                    "decoded_payload": {"avgHumid": 13.2, "avgTemperature": 13.8}
                }
            }
        });
        let v = unwrap_envelope(&env);
        assert_eq!(v["avgHumid"], 13.2);
        assert_eq!(v["avgTemperature"], 13.8);
    }

    #[test]
    fn pick_device_reads_as_up_data_forward_envelope() {
        let env = json!({
            "data": {"end_device_ids": {"device_id": "eui-a8404107bf5e6409"}}
        });
        assert_eq!(pick_device(&env).unwrap(), "eui-a8404107bf5e6409");
    }

    #[test]
    fn pick_device_reads_identifiers_envelope() {
        let env = json!({
            "identifiers": [
                {"device_ids": {"device_id": "eui-a8404131af5e6451"}}
            ]
        });
        assert_eq!(pick_device(&env).unwrap(), "eui-a8404131af5e6451");
    }

    #[test]
    fn pick_device_iterates_identifiers_when_device_ids_is_not_first() {
        let env = json!({
            "identifiers": [
                {"application_ids": {"application_id": "tbz-lns"}},
                {"gateway_ids": {"gateway_id": "gw-1"}},
                {"device_ids": {"device_id": "eui-a8404131af5e6451"}}
            ]
        });
        assert_eq!(pick_device(&env).unwrap(), "eui-a8404131af5e6451");
    }

    #[test]
    fn pick_device_falls_back_to_flat_device_field() {
        assert_eq!(pick_device(&json!({"device": "eui-x"})).unwrap(), "eui-x");
        assert_eq!(
            pick_device(&json!({"deviceName": "eui-y"})).unwrap(),
            "eui-y"
        );
    }

    #[test]
    fn pick_device_reports_missing_id() {
        let err = pick_device(&json!({})).unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(ref s) if s == "missing device id"));
    }

    #[test]
    fn build_eco_drizzler_normalises_watermarks() {
        let body = json!({
            "device": "eui-eco",
            "battery": 3.6,
            "humidity": 0.4,
            "temperature": 18.0,
            "latitude": 53.55,
            "longitude": 9.99,
            "watermarks": [
                {"depth": 30, "resistance": 0, "centibar": 25},
                {"depth": 60, "resistance": 0, "centibar": 30},
                {"depth": 90, "resistance": 0, "centibar": 35},
            ]
        });
        let model = eco_drizzler_model();
        let ingest = build_eco_drizzler(&model, &body, SensorId::new("eui-eco").unwrap()).unwrap();
        assert_eq!(ingest.normalized.len(), 3);
        assert!(matches!(ingest.typed, SensorReadings::Watermarks(ref w) if w.len() == 3));
    }

    #[test]
    fn build_ges_1000_emits_volumetrics_and_skips_unknown_abilities() {
        let body = json!({
            "device": "eui-ges",
            "battery": 3.7,
            "readings": [
                {"ability": "soil_moisture", "depth": 40, "value": 42.0},
                {"ability": "soil_moisture", "depth": 80, "value": 25.0},
                {"ability": "salinity",      "depth": 40, "value": 1.0},
                {"ability": "soil_moisture", "depth": 99, "value": 9.0},
            ]
        });
        let model = ges_1000_model();
        let ingest = build_ges_1000(&model, &body, SensorId::new("eui-ges").unwrap()).unwrap();
        assert_eq!(ingest.normalized.len(), 2);
        match ingest.typed {
            SensorReadings::Volumetrics(ref v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0].depth_cm, 40);
                assert_eq!(v[1].depth_cm, 80);
            }
            _ => panic!("expected Volumetrics"),
        }
    }

    #[test]
    fn build_ges_1000_parses_ttn_sensor_pair() {
        let body = json!({
            "avgHumid": 47.5,
            "avgTemperature": 18.6,
            "battery_v": 3.582,
            "sensor1": { "humidity": 43.5, "id": 1, "temperature": 18.5 },
            "sensor2": { "humidity": 51.6, "id": 2, "temperature": 18.7 }
        });
        let model = ges_1000_model();
        let ingest = build_ges_1000(
            &model,
            &body,
            SensorId::new("eui-a8404131af5e6451").unwrap(),
        )
        .unwrap();
        assert_eq!(ingest.normalized.len(), 5);
        match ingest.typed {
            SensorReadings::Volumetrics(ref v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0].depth_cm, 40);
                assert_eq!(v[0].moisture_percent, 43.5);
                assert_eq!(v[1].depth_cm, 80);
                assert_eq!(v[1].moisture_percent, 51.6);
            }
            _ => panic!("expected Volumetrics"),
        }
    }

    #[test]
    fn decode_ttn_ges_1000_maps_probe_id_to_depth_regardless_of_key_order() {
        // Inverted key/id order would have flipped depths under positional mapping.
        let body = json!({
            "sensor1": { "humidity": 51.6, "id": 2, "temperature": 18.7 },
            "sensor2": { "humidity": 43.5, "id": 1, "temperature": 18.5 }
        });
        let payload =
            decode_ttn_ges_1000(&body, &SensorId::new("eui-a8404131af5e6451").unwrap()).unwrap();
        let moisture: Vec<_> = payload
            .readings
            .iter()
            .filter(|r| r.ability == "soil_moisture")
            .collect();
        assert_eq!(moisture.len(), 2);
        assert_eq!(moisture[0].depth, 80);
        assert_eq!(moisture[0].value, 51.6);
        assert_eq!(moisture[1].depth, 40);
        assert_eq!(moisture[1].value, 43.5);
    }

    #[test]
    fn decode_ttn_ges_1000_skips_probe_with_unknown_id() {
        let body = json!({
            "sensor1": { "humidity": 43.5, "id": 1, "temperature": 18.5 },
            "sensor2": { "humidity": 99.9, "id": 7, "temperature": 99.9 }
        });
        let payload = decode_ttn_ges_1000(&body, &SensorId::new("eui-ges").unwrap()).unwrap();
        assert_eq!(payload.readings.len(), 2);
        assert!(payload.readings.iter().all(|r| r.depth == 40));
    }

    #[test]
    fn decode_ttn_ges_1000_rejects_empty_payload() {
        let body = json!({ "avgHumid": 0.0, "avgTemperature": 0.0 });
        let err = decode_ttn_ges_1000(&body, &SensorId::new("eui-x").unwrap()).unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(_)));
    }

    #[test]
    fn decode_ttn_ges_1000_rejects_battery_only_payload() {
        let body = json!({ "battery_v": 3.58 });
        let err = decode_ttn_ges_1000(&body, &SensorId::new("eui-x").unwrap()).unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(_)));
    }

    #[test]
    fn build_ingest_rejects_unknown_model() {
        let model = SensorModel {
            id: Id::new_v7(),
            name: SensorModelName::new("UnknownModel").unwrap(),
            description: None,
            abilities: vec![],
        };
        let err =
            build_ingest(&model, &json!({"device": "x"}), SensorId::new("x").unwrap()).unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(_)));
    }
}
