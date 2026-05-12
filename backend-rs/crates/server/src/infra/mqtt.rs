//! MQTT ingestor: subscribes to a broker topic, looks up the sensor model
//! to pick the right wire-format parser, and forwards the normalized reading
//! to [`SensorService::ingest_reading`].
//!
//! Disabled by default; flip `mqtt.enabled = true` (and set `broker_url` /
//! `topic`) to start the background task. The task reconnects automatically
//! through `rumqttc`'s `EventLoop` and survives broker outages.

use std::{str::FromStr, sync::Arc, time::Duration};

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
        payload::{EcoDrizzlerPayload, GenericReadingPayload},
        repository::NormalizedValue,
    },
    sensor_model::{SensorAbilityName, SensorModel},
};

/// Spawns the MQTT subscriber as a tokio task. Returns `Ok(())` if disabled
/// or if the task started successfully. The task itself logs and recovers
/// from connection errors; the caller does not await its completion.
pub fn spawn(
    settings: MqttSettings,
    sensor_service: Arc<SensorService>,
) -> Result<(), MqttSubscriberError> {
    if !settings.enabled {
        tracing::info!("mqtt subscriber disabled due to config (mqtt.enabled = false)");
        return Ok(());
    }
    if settings.broker_url.is_empty() || settings.topic.is_empty() {
        return Err(MqttSubscriberError::MissingConfig);
    }

    let (client, eventloop) = build_client(&settings)?;
    let topic = settings.topic.clone();

    tokio::spawn(async move {
        if let Err(e) = client.subscribe(&topic, QoS::AtLeastOnce).await {
            tracing::error!(error = %e, %topic, "mqtt subscribe failed");
            return;
        }
        run_event_loop(eventloop, sensor_service).await;
    });

    Ok(())
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

async fn run_event_loop(mut eventloop: EventLoop, sensor_service: Arc<SensorService>) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(pub_pkt))) => {
                if let Err(e) = handle_publish(&pub_pkt.payload, &sensor_service).await {
                    tracing::warn!(error = %e, "mqtt message dropped");
                }
            }
            Ok(_) => {}
            Err(e) => {
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
    let body = unwrap_envelope(raw)?;
    let device = pick_device(&body)?;
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

/// Decodes the TTN-style envelope (`{"uplink_message":{"decoded_payload":…}}`)
/// or returns the raw body if no envelope is present — useful for direct
/// publish testing.
fn unwrap_envelope(raw: &[u8]) -> Result<Value, MqttSubscriberError> {
    let v: Value =
        serde_json::from_slice(raw).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;
    if let Some(decoded) = v
        .get("uplink_message")
        .and_then(|u| u.get("decoded_payload"))
    {
        return Ok(decoded.clone());
    }
    Ok(v)
}

fn pick_device(v: &Value) -> Result<String, MqttSubscriberError> {
    v.get("device")
        .and_then(|x| x.as_str())
        .map(str::to_owned)
        .or_else(|| {
            v.get("deviceName")
                .and_then(|x| x.as_str())
                .map(str::to_owned)
        })
        .ok_or_else(|| MqttSubscriberError::Decode("missing device id".into()))
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
    let payload: GenericReadingPayload = serde_json::from_value(body.clone())
        .map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;
    let raw_payload =
        serde_json::to_value(&payload).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;

    let mut normalized = Vec::with_capacity(payload.readings.len());
    let mut volumetrics = Vec::new();
    for r in &payload.readings {
        let name = match SensorAbilityName::from_str(&r.ability) {
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(error = %e, ability = %r.ability, "dropping reading with unknown ability");
                continue;
            }
        };
        let Some(model_ability_id) = model.ability_id_for(name, r.depth) else {
            tracing::warn!(
                ability = %r.ability,
                depth = r.depth,
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

fn normalize_eco_drizzler(model: &SensorModel, payload: &EcoDrizzlerPayload) -> Vec<NormalizedValue> {
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
        sensor_model::{
            SensorAbility, SensorAbilityUnit, SensorModelAbility, SensorModelName,
        },
    };
    use serde_json::json;

    fn eco_drizzler_model() -> SensorModel {
        let st = |id, depth_cm| SensorModelAbility {
            id,
            ability: SensorAbility {
                id: 1,
                name: SensorAbilityName::SoilTension,
                unit: SensorAbilityUnit::Centibar,
            },
            depth_cm,
        };
        SensorModel {
            id: Id::new(1),
            name: SensorModelName::new("EcoDrizzler").unwrap(),
            description: None,
            abilities: vec![st(1, 30), st(2, 60), st(3, 90)],
        }
    }

    fn ges_1000_model() -> SensorModel {
        let moisture = |id, depth_cm| SensorModelAbility {
            id,
            ability: SensorAbility {
                id: 2,
                name: SensorAbilityName::SoilMoisture,
                unit: SensorAbilityUnit::Percent,
            },
            depth_cm,
        };
        SensorModel {
            id: Id::new(2),
            name: SensorModelName::new("GES-1000").unwrap(),
            description: None,
            abilities: vec![moisture(10, 30), moisture(11, 60), moisture(12, 90)],
        }
    }

    #[test]
    fn unwrap_envelope_handles_ttn_and_flat() {
        let env = json!({"uplink_message": {"decoded_payload": {"device": "eui-1"}}});
        let v = unwrap_envelope(&serde_json::to_vec(&env).unwrap()).unwrap();
        assert_eq!(v["device"], "eui-1");

        let flat = json!({"device": "eui-2"});
        let v = unwrap_envelope(&serde_json::to_vec(&flat).unwrap()).unwrap();
        assert_eq!(v["device"], "eui-2");
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
                {"ability": "soil_moisture", "depth": 30, "value": 42.0},
                {"ability": "soil_moisture", "depth": 90, "value": 25.0},
                {"ability": "salinity",      "depth": 30, "value": 1.0},
                {"ability": "soil_moisture", "depth": 99, "value": 9.0},
            ]
        });
        let model = ges_1000_model();
        let ingest = build_ges_1000(&model, &body, SensorId::new("eui-ges").unwrap()).unwrap();
        assert_eq!(ingest.normalized.len(), 2);
        match ingest.typed {
            SensorReadings::Volumetrics(ref v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0].depth_cm, 30);
                assert_eq!(v[1].depth_cm, 90);
            }
            _ => panic!("expected Volumetrics"),
        }
    }

    #[test]
    fn build_ingest_rejects_unknown_model() {
        let model = SensorModel {
            id: Id::new(99),
            name: SensorModelName::new("UnknownModel").unwrap(),
            description: None,
            abilities: vec![],
        };
        let err = build_ingest(
            &model,
            &json!({"device": "x"}),
            SensorId::new("x").unwrap(),
        )
        .unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(_)));
    }
}
