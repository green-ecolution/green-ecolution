use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use chrono::Utc;
use domain::{
    Id, RepositoryError,
    data_export::{ExportCatalog, ExportState, ExportThing, StateValue},
    organization::Organization,
    shared::coordinates::Coordinate,
    target::TargetRef,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use waste_domain::{
    export::{FILL_LEVEL, FILL_STATE},
    fill::{FillMeasurement, derive_fill_state},
};

use crate::{KEY, service::fill_rules::FillRulesResolver, targets::BIN_KIND};

pub struct WasteExportCatalog {
    pool: PgPool,
    rules: Arc<FillRulesResolver>,
}

impl WasteExportCatalog {
    pub fn new(pool: PgPool, rules: Arc<FillRulesResolver>) -> Self {
        Self { pool, rules }
    }
}

#[async_trait::async_trait]
impl ExportCatalog for WasteExportCatalog {
    #[tracing::instrument(level = "debug", skip_all, fields(orgs = orgs.len()))]
    async fn snapshot(
        &self,
        orgs: &BTreeSet<Id<Organization>>,
    ) -> Result<Vec<ExportThing>, RepositoryError> {
        let ids: Vec<Uuid> = orgs.iter().map(|o| o.value()).collect();
        let bins = sqlx::query!(
            r#"SELECT id, number, latitude, longitude, volume_liters, depth_cm, organization_id,
                      fill_fraction, fill_measured_at, last_emptied_at
               FROM waste_bins WHERE organization_id = ANY($1)"#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut rules = HashMap::new();
        for org in bins
            .iter()
            .map(|b| b.organization_id)
            .collect::<BTreeSet<_>>()
        {
            rules.insert(org, self.rules.rules_for(Id::new(org)).await?);
        }

        let now = Utc::now();
        let mut things = Vec::with_capacity(bins.len());
        for bin in bins {
            let rules = rules[&bin.organization_id];
            let fill =
                bin.fill_fraction
                    .zip(bin.fill_measured_at)
                    .map(|(fraction, measured_at)| FillMeasurement {
                        fraction,
                        measured_at,
                    });
            let state = derive_fill_state(
                fill,
                bin.last_emptied_at,
                rules.threshold,
                now,
                rules.stale_after,
            );
            let mut states = Vec::with_capacity(2);
            if let Some(fill) = fill {
                states.push(ExportState {
                    property: FILL_LEVEL,
                    value: StateValue::Measurement(fill.fraction * 100.0),
                    observed_at: fill.measured_at,
                });
            }
            states.push(ExportState {
                property: FILL_STATE,
                value: StateValue::Category(state.as_str().to_owned()),
                observed_at: now,
            });
            things.push(ExportThing {
                target: TargetRef::new(KEY, BIN_KIND, bin.id).map_err(RepositoryError::from)?,
                name: format!("Behälter {}", bin.number),
                description: format!("{} l", bin.volume_liters),
                attributes: BTreeMap::from([
                    ("number".into(), json!(bin.number)),
                    ("volume_liters".into(), json!(bin.volume_liters)),
                    ("depth_cm".into(), json!(bin.depth_cm)),
                    (
                        "organization_id".into(),
                        json!(bin.organization_id.to_string()),
                    ),
                ]),
                coordinate: Coordinate::new(bin.latitude, bin.longitude).ok(),
                states,
            });
        }
        Ok(things)
    }
}
