use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use crate::{
    Id, RepositoryError, organization::Organization, shared::coordinates::Coordinate,
    target::TargetRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    Measurement,
    Category,
}

impl ObservationKind {
    pub fn observation_type(self) -> &'static str {
        match self {
            Self::Measurement => {
                "http://www.opengis.net/def/observationType/OGC-OM/2.0/OM_Measurement"
            }
            Self::Category => {
                "http://www.opengis.net/def/observationType/OGC-OM/2.0/OM_CategoryObservation"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportUnit {
    pub name: &'static str,
    pub symbol: &'static str,
    pub definition: &'static str,
}

/// A quantity the export writes; `key` is part of the remote keys, so renaming
/// one orphans the datastreams already written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportProperty {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub kind: ObservationKind,
    pub unit: Option<ExportUnit>,
}

pub const PERCENT: ExportUnit = ExportUnit {
    name: "percent",
    symbol: "%",
    definition: "http://unitsofmeasure.org/ucum.html",
};

#[derive(Debug, Clone, PartialEq)]
pub enum StateValue {
    Category(String),
    Measurement(f64),
}

impl StateValue {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Category(c) => json!(c),
            Self::Measurement(m) => json!(m),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportState {
    pub property: ExportProperty,
    pub value: StateValue,
    pub observed_at: DateTime<Utc>,
}

/// One module object as the platform sees it. `attributes` are the module's
/// own fields; the sink adds the identifying properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportThing {
    pub target: TargetRef,
    pub name: String,
    pub description: String,
    pub attributes: BTreeMap<String, Value>,
    pub coordinate: Option<Coordinate>,
    pub states: Vec<ExportState>,
}

pub fn ge_key(target: &TargetRef) -> String {
    format!(
        "{}:{}:{}",
        target.module.as_str(),
        target.kind.as_str(),
        target.id
    )
}

impl ExportThing {
    pub fn key(&self) -> String {
        ge_key(&self.target)
    }

    pub fn content_fingerprint(&self) -> String {
        fingerprint(&json!({
            "name": self.name,
            "description": self.description,
            "attributes": self.attributes,
        }))
    }

    pub fn location_fingerprint(&self) -> Option<String> {
        self.coordinate
            .map(|c| fingerprint(&json!([c.latitude(), c.longitude()])))
    }
}

/// FNV-1a over the canonical JSON (serde_json keeps object keys sorted). Not
/// `DefaultHasher`: the value is stored, so it must not change with the toolchain.
fn fingerprint(value: &Value) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in value.to_string().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[async_trait::async_trait]
pub trait ExportCatalog: Send + Sync {
    /// Every exportable object of the module in these organizations.
    async fn snapshot(
        &self,
        orgs: &BTreeSet<Id<Organization>>,
    ) -> Result<Vec<ExportThing>, RepositoryError>;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::coordinates::Coordinate;
    use serde_json::json;
    use uuid::Uuid;

    fn thing() -> ExportThing {
        let id = Uuid::parse_str("0192f3a0-0000-7000-8000-000000000001").unwrap();
        ExportThing {
            target: TargetRef::new("irrigation", "tree", id).unwrap(),
            name: "Baum 12".into(),
            description: "Linde".into(),
            attributes: BTreeMap::from([("species".to_string(), json!("Linde"))]),
            coordinate: Some(Coordinate::new(54.79, 9.43).unwrap()),
            states: vec![],
        }
    }

    #[test]
    fn the_key_names_module_kind_and_id() {
        assert_eq!(
            thing().key(),
            "irrigation:tree:0192f3a0-0000-7000-8000-000000000001"
        );
    }

    #[test]
    fn fingerprints_are_stable_and_separate_content_from_location() {
        let a = thing();
        let mut moved = thing();
        moved.coordinate = Some(Coordinate::new(54.80, 9.43).unwrap());
        let mut renamed = thing();
        renamed.name = "Baum 13".into();

        assert_eq!(a.content_fingerprint(), thing().content_fingerprint());
        assert_eq!(a.content_fingerprint(), moved.content_fingerprint());
        assert_ne!(a.location_fingerprint(), moved.location_fingerprint());
        assert_ne!(a.content_fingerprint(), renamed.content_fingerprint());
        assert_eq!(a.location_fingerprint(), renamed.location_fingerprint());
    }

    #[test]
    fn a_thing_without_coordinate_has_no_location_fingerprint() {
        let mut t = thing();
        t.coordinate = None;
        assert_eq!(t.location_fingerprint(), None);
    }

    #[test]
    fn state_values_serialize_as_plain_json() {
        assert_eq!(StateValue::Category("good".into()).to_json(), json!("good"));
        assert_eq!(StateValue::Measurement(42.5).to_json(), json!(42.5));
    }

    #[test]
    fn category_observations_have_their_om_type() {
        assert!(
            ObservationKind::Category
                .observation_type()
                .ends_with("OM_CategoryObservation")
        );
    }
}
