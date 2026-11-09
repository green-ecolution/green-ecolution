//! What the irrigation module writes to an urban data platform.

use domain::data_export::{ExportProperty, ObservationKind};

pub const WATERING_STATUS: ExportProperty = ExportProperty {
    key: "watering_status",
    label: "Bewässerungsstatus",
    description: "Derived watering status: good, moderate, bad, just_watered or unknown",
    kind: ObservationKind::Category,
    unit: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_watering_status_is_a_category_without_unit() {
        assert_eq!(WATERING_STATUS.kind, ObservationKind::Category);
        assert!(WATERING_STATUS.unit.is_none());
    }
}
