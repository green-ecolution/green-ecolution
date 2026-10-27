//! Parsers that modules contribute for their own sensor models. The MQTT
//! ingestor asks this registry after its built-in parsers.

use std::{collections::BTreeMap, sync::Arc};

use domain::{sensor::SensorId, sensor_model::SensorModel};

use crate::service::sensor_service::ReadingIngest;

pub trait SensorPayloadParser: Send + Sync {
    /// Exact `sensor_models.name` this parser decodes.
    fn model_name(&self) -> &'static str;

    /// `normalized` must carry the model's ability values (the plausibility filter flags
    /// those); typed readings must use the model ability's depth.
    fn parse(
        &self,
        model: &SensorModel,
        body: &serde_json::Value,
        sensor_id: SensorId,
    ) -> Result<ReadingIngest, PayloadError>;
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct PayloadError(pub String);

#[derive(Debug, thiserror::Error)]
#[error("sensor model {0} has more than one parser")]
pub struct DuplicateParser(pub String);

pub struct SensorParserRegistry {
    parsers: BTreeMap<&'static str, Arc<dyn SensorPayloadParser>>,
}

impl SensorParserRegistry {
    /// Models the MQTT ingestor decodes itself; a module may not claim them.
    pub const BUILT_IN: &'static [&'static str] = &["EcoDrizzler", "GES-1000"];

    pub fn new(parsers: Vec<Arc<dyn SensorPayloadParser>>) -> Result<Self, DuplicateParser> {
        let mut map = BTreeMap::new();
        for parser in parsers {
            let name = parser.model_name();
            if Self::BUILT_IN.contains(&name) || map.insert(name, parser).is_some() {
                return Err(DuplicateParser(name.to_string()));
            }
        }
        Ok(Self { parsers: map })
    }

    pub fn empty() -> Self {
        Self {
            parsers: BTreeMap::new(),
        }
    }

    pub fn get(&self, model_name: &str) -> Option<&Arc<dyn SensorPayloadParser>> {
        self.parsers.get(model_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Named(&'static str);
    impl SensorPayloadParser for Named {
        fn model_name(&self) -> &'static str {
            self.0
        }
        fn parse(
            &self,
            _: &SensorModel,
            _: &serde_json::Value,
            _: SensorId,
        ) -> Result<ReadingIngest, PayloadError> {
            Err(PayloadError("unused".into()))
        }
    }

    #[test]
    fn a_module_parser_is_found_by_model_name() {
        let registry = SensorParserRegistry::new(vec![Arc::new(Named("Level-1"))]).unwrap();
        assert!(registry.get("Level-1").is_some());
        assert!(registry.get("Level-2").is_none());
    }

    #[test]
    fn two_parsers_for_one_model_are_refused() {
        let err = SensorParserRegistry::new(vec![Arc::new(Named("X")), Arc::new(Named("X"))])
            .err()
            .expect("duplicate must fail");
        assert_eq!(err.0, "X");
    }

    #[test]
    fn a_module_cannot_take_over_a_built_in_model() {
        assert!(SensorParserRegistry::new(vec![Arc::new(Named("GES-1000"))]).is_err());
    }
}
