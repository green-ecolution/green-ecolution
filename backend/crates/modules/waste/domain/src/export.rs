//! What the waste module writes to an urban data platform.

use domain::data_export::{ExportProperty, ObservationKind, PERCENT};

pub const FILL_LEVEL: ExportProperty = ExportProperty {
    key: "fill_level",
    label: "Füllstand",
    description: "Measured fill level of the bin in percent",
    kind: ObservationKind::Measurement,
    unit: Some(PERCENT),
};

pub const FILL_STATE: ExportProperty = ExportProperty {
    key: "fill_state",
    label: "Füllzustand",
    description: "Derived fill state: empty, normal, full or unknown",
    kind: ObservationKind::Category,
    unit: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fill_level_is_in_percent_and_the_fill_state_has_no_unit() {
        assert_eq!(FILL_LEVEL.unit.map(|u| u.symbol), Some("%"));
        assert!(FILL_STATE.unit.is_none());
    }
}
