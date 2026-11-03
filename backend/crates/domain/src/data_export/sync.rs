use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::{
    ExportProperty, ExportRunStats, ExportTargetConfig, ExportThing, StateValue, StoredExportTarget,
};
use crate::{Id, RepositoryError, organization::Organization};

pub const SENSOR_KEY: &str = "sensor:green-ecolution";

pub fn location_key(thing_key: &str) -> String {
    format!("location:{thing_key}")
}

pub fn property_key(property: &ExportProperty) -> String {
    format!("property:{}", property.key)
}

pub fn state_stream_key(property: &ExportProperty, thing_key: &str) -> String {
    format!("state:{}:{thing_key}", property.key)
}

/// A SensorThings `@iot.id`; FROST defaults to numbers, other servers use strings.
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteId(pub Value);

impl RemoteId {
    pub fn from_location(header: &str) -> Option<Self> {
        let open = header.rfind('(')?;
        let inner = header[open + 1..].strip_suffix(')')?;
        if let Some(quoted) = inner.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            return Some(Self(Value::String(quoted.replace("''", "'"))));
        }
        inner.parse::<i64>().ok().map(|n| Self(Value::from(n)))
    }
    pub fn from_json(value: &Value) -> Option<Self> {
        match value {
            Value::String(_) => Some(Self(value.clone())),
            Value::Number(n) => n.as_i64().map(|n| Self(Value::from(n))),
            _ => None,
        }
    }

    /// Ids are remote input; the encoding keeps one from leaving its path segment.
    pub fn path_segment(&self) -> String {
        match &self.0 {
            Value::String(s) => {
                let mut out = String::from("('");
                for byte in s.bytes() {
                    match byte {
                        b'\'' => out.push_str("''"),
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                            out.push(byte as char)
                        }
                        _ => out.push_str(&format!("%{byte:02X}")),
                    }
                }
                out.push_str("')");
                out
            }
            other => format!("({other})"),
        }
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportEntity {
    Thing,
    Location,
    Sensor,
    ObservedProperty,
    Datastream,
}

impl ExportEntity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Thing => "thing",
            Self::Location => "location",
            Self::Sensor => "sensor",
            Self::ObservedProperty => "observed_property",
            Self::Datastream => "datastream",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "thing" => Some(Self::Thing),
            "location" => Some(Self::Location),
            "sensor" => Some(Self::Sensor),
            "observed_property" => Some(Self::ObservedProperty),
            "datastream" => Some(Self::Datastream),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportRef {
    pub ge_key: String,
    pub entity: ExportEntity,
    pub remote_id: RemoteId,
    pub content_hash: Option<String>,
    pub last_value: Option<Value>,
    pub deleted_marked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunResult {
    Succeeded(ExportRunStats),
    /// `error` must not contain secrets; it is shown in the settings UI.
    Failed {
        error: String,
        base_interval_secs: i64,
    },
}

#[async_trait::async_trait]
pub trait ExportRefReader: Send + Sync {
    async fn refs(
        &self,
        org: Id<Organization>,
    ) -> Result<HashMap<String, ExportRef>, RepositoryError>;
}

#[async_trait::async_trait]
pub trait ExportRefWriter: Send + Sync {
    /// Writes only while the target still has this generation (`activated_at`), so a
    /// pass that outlived a base-URL change writes nothing. A stale write is not an error.
    async fn put_ref(
        &self,
        org: Id<Organization>,
        generation: DateTime<Utc>,
        export_ref: &ExportRef,
    ) -> Result<(), RepositoryError>;
}

#[async_trait::async_trait]
pub trait ExportRunStore: Send + Sync {
    /// Every organization with a target, enabled or not: they bound the scopes.
    async fn target_orgs(&self) -> Result<BTreeSet<Id<Organization>>, RepositoryError>;
    /// Enabled, not leased and past their backoff.
    async fn due(&self) -> Result<Vec<Id<Organization>>, RepositoryError>;
    /// `None` when another replica holds the lease or the target is not due.
    async fn try_lease(
        &self,
        org: Id<Organization>,
        lease_secs: i64,
    ) -> Result<Option<StoredExportTarget>, RepositoryError>;
    /// Writes only while the target still has this generation (`activated_at`), so a
    /// pass that outlived a base-URL change writes nothing. A stale write is not an error.
    async fn finish(
        &self,
        org: Id<Organization>,
        generation: DateTime<Utc>,
        result: &RunResult,
    ) -> Result<(), RepositoryError>;
    async fn cursor(&self, org: Id<Organization>) -> Result<Option<Uuid>, RepositoryError>;
    /// Like `finish`, writes only while the target still has this generation.
    async fn advance_cursor(
        &self,
        org: Id<Organization>,
        generation: DateTime<Utc>,
        cursor: Uuid,
    ) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SinkError {
    #[error("the target is not on the instance allowlist")]
    UrlNotAllowed,
    #[error("target unreachable: {0}")]
    Unreachable(String),
    #[error("target refused the credentials (HTTP {0})")]
    Unauthorized(u16),
    #[error("target rejected the request (HTTP {status})")]
    Rejected {
        status: u16,
        /// Start of the remote answer, for debugging only. Never persist or log it:
        /// a proxy echoing the request may put the credentials in there.
        detail: String,
    },
    #[error("unexpected answer from the target: {0}")]
    Malformed(String),
}

impl SinkError {
    /// A rejected single entity is skipped; everything else means the target
    /// cannot be worked with right now.
    pub fn aborts_pass(&self) -> bool {
        !matches!(self, Self::Rejected { .. })
    }
}

pub struct StateStream<'a> {
    pub ge_key: &'a str,
    pub thing: &'a RemoteId,
    pub thing_name: &'a str,
    pub property: &'a ExportProperty,
    pub sensor: &'a RemoteId,
    pub observed_property: &'a RemoteId,
}

#[async_trait::async_trait]
pub trait SensorThingsSink: Send + Sync {
    /// Checks the URL policy and authenticates once for a sync pass.
    async fn open(&self, config: &ExportTargetConfig) -> Result<Box<dyn SinkSession>, SinkError>;
}

#[async_trait::async_trait]
pub trait SinkSession: Send + Sync {
    async fn create_thing(&self, thing: &ExportThing) -> Result<(RemoteId, bool), SinkError>;
    async fn update_thing(&self, id: &RemoteId, thing: &ExportThing) -> Result<(), SinkError>;
    async fn move_thing(&self, id: &RemoteId, thing: &ExportThing) -> Result<(), SinkError>;
    async fn mark_thing_deleted(
        &self,
        id: &RemoteId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), SinkError>;
    async fn ensure_sensor(&self) -> Result<RemoteId, SinkError>;
    async fn ensure_observed_property(
        &self,
        property: &ExportProperty,
    ) -> Result<RemoteId, SinkError>;
    async fn ensure_state_datastream(
        &self,
        stream: &StateStream<'_>,
    ) -> Result<RemoteId, SinkError>;
    async fn post_observation(
        &self,
        datastream: &RemoteId,
        value: &StateValue,
        phenomenon_time: DateTime<Utc>,
    ) -> Result<(), SinkError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_export::WATERING_STATUS;
    use serde_json::json;

    #[test]
    fn reads_numeric_and_string_ids_from_a_location_header() {
        assert_eq!(
            RemoteId::from_location("http://x/FROST-Server/v1.1/Things(42)"),
            Some(RemoteId(json!(42)))
        );
        assert_eq!(
            RemoteId::from_location("http://x/v1.1/Things('a''b')"),
            Some(RemoteId(json!("a'b")))
        );
        assert_eq!(RemoteId::from_location("http://x/v1.1/Things"), None);
    }

    #[test]
    fn malformed_location_ids_are_refused() {
        for h in [
            "x/Things(abc)",
            "x/Things(42",
            "x/Things()",
            "x/Things('a(b)')",
        ] {
            assert_eq!(RemoteId::from_location(h), None, "{h}");
        }
    }

    #[test]
    fn json_ids_must_be_integers_or_strings() {
        assert_eq!(RemoteId::from_json(&json!(3)), Some(RemoteId(json!(3))));
        assert_eq!(RemoteId::from_json(&json!("a")), Some(RemoteId(json!("a"))));
        assert_eq!(RemoteId::from_json(&json!(1.5)), None);
        assert_eq!(RemoteId::from_json(&json!({"x": 1})), None);
        assert_eq!(RemoteId::from_json(&json!(null)), None);
    }

    #[test]
    fn renders_ids_as_odata_path_segments() {
        assert_eq!(RemoteId(json!(7)).path_segment(), "(7)");
        assert_eq!(RemoteId(json!("a'b")).path_segment(), "('a''b')");
        assert_eq!(RemoteId(json!("a/b?c")).path_segment(), "('a%2Fb%3Fc')");
        assert_eq!(RemoteId(json!("../x")).path_segment(), "('..%2Fx')");
    }

    #[test]
    fn entities_round_trip_through_their_names() {
        for e in [
            ExportEntity::Thing,
            ExportEntity::Location,
            ExportEntity::Sensor,
            ExportEntity::ObservedProperty,
            ExportEntity::Datastream,
        ] {
            assert_eq!(ExportEntity::parse(e.as_str()), Some(e));
        }
    }

    #[test]
    fn keys_are_namespaced() {
        assert_eq!(
            location_key("irrigation:tree:1"),
            "location:irrigation:tree:1"
        );
        assert_eq!(property_key(&WATERING_STATUS), "property:watering_status");
        assert_eq!(
            state_stream_key(&WATERING_STATUS, "irrigation:tree:1"),
            "state:watering_status:irrigation:tree:1"
        );
    }

    #[test]
    fn only_rejections_keep_a_pass_running() {
        assert!(
            !SinkError::Rejected {
                status: 400,
                detail: "x".into()
            }
            .aborts_pass()
        );
        assert!(SinkError::Unreachable("x".into()).aborts_pass());
        assert!(SinkError::Unauthorized(401).aborts_pass());
        assert!(SinkError::UrlNotAllowed.aborts_pass());
    }
}
