//! MQTT ingestor: subscribes to a broker topic, parses TTN-style uplink
//! payloads, and forwards them to [`SensorService::handle_message`].
//!
//! Disabled by default; flip `mqtt.enabled = true` (and set `broker_url` /
//! `topic`) to start the background task. The task reconnects automatically
//! through `rumqttc`'s `EventLoop` and survives broker outages.

use std::{sync::Arc, time::Duration};

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS, Transport};
use secrecy::ExposeSecret;
use serde_json::Value;
use url::Url;

use crate::{configuration::MqttSettings, service::sensor_service::SensorService};
use domain::sensor::data::{MqttPayload, Watermark};

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
    let payload = parse_payload(raw)?;
    sensor_service
        .handle_message(payload)
        .await
        .map_err(|e| MqttSubscriberError::Service(e.to_string()))
}

/// Decodes the TTN-style uplink envelope used by the Go backend:
/// `{"uplink_message":{"decoded_payload":{...}}}`. Falls back to interpreting
/// the body as a flat [`MqttPayload`] when the envelope is absent — useful
/// for direct-publish testing.
fn parse_payload(raw: &[u8]) -> Result<MqttPayload, MqttSubscriberError> {
    let v: Value =
        serde_json::from_slice(raw).map_err(|e| MqttSubscriberError::Decode(e.to_string()))?;
    if let Some(decoded) = v
        .get("uplink_message")
        .and_then(|u| u.get("decoded_payload"))
    {
        return ttn_decoded_to_payload(decoded);
    }
    serde_json::from_value(v).map_err(|e| MqttSubscriberError::Decode(e.to_string()))
}

fn ttn_decoded_to_payload(decoded: &Value) -> Result<MqttPayload, MqttSubscriberError> {
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
    Ok(MqttPayload {
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
    use serde_json::json;

    #[test]
    fn parses_ttn_envelope() {
        let raw = json!({
            "uplink_message": {
                "decoded_payload": {
                    "deviceName": "eui-deadbeef",
                    "batteryVoltage": 3.6,
                    "waterContent": 0.4,
                    "temperature": 18.0,
                    "latitude": 53.55,
                    "longitude": 9.99,
                    "WM30_Resistance": 1000,
                    "WM30_CB": 28,
                    "WM60_Resistance": 1100,
                    "WM60_CB": 30,
                    "WM90_Resistance": 1200,
                    "WM90_CB": 35,
                }
            }
        });
        let payload = parse_payload(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(payload.device, "eui-deadbeef");
        assert_eq!(payload.watermarks.len(), 3);
        assert_eq!(payload.watermarks[0].depth, 30);
        assert_eq!(payload.watermarks[0].centibar, 28);
        assert_eq!(payload.watermarks[2].centibar, 35);
    }

    #[test]
    fn parses_flat_payload_fallback() {
        let raw = json!({
            "device": "eui-flat",
            "battery": 3.7,
            "humidity": 0.3,
            "temperature": 17.0,
            "latitude": 53.55,
            "longitude": 9.99,
            "watermarks": [
                {"depth": 30, "resistance": 0, "centibar": 5},
                {"depth": 60, "resistance": 0, "centibar": 5},
                {"depth": 90, "resistance": 0, "centibar": 5},
            ]
        });
        let payload = parse_payload(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(payload.device, "eui-flat");
        assert_eq!(payload.watermarks.len(), 3);
    }

    #[test]
    fn rejects_missing_field() {
        let raw = json!({
            "uplink_message": {
                "decoded_payload": {
                    "deviceName": "x"
                    // missing all other fields
                }
            }
        });
        let err = parse_payload(&serde_json::to_vec(&raw).unwrap()).unwrap_err();
        assert!(matches!(err, MqttSubscriberError::Decode(_)));
    }
}
