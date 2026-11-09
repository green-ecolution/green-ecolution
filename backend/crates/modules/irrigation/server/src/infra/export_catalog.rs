use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use domain::{
    Id, RepositoryError,
    data_export::{ExportCatalog, ExportState, ExportThing, StateValue},
    organization::Organization,
    shared::coordinates::Coordinate,
    target::TargetRef,
};
use irrigation_domain::export::WATERING_STATUS;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    KEY,
    targets::{CLUSTER_KIND, TREE_KIND},
};

pub struct IrrigationExportCatalog {
    pool: PgPool,
}

impl IrrigationExportCatalog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn target(kind: &str, id: Uuid) -> Result<TargetRef, RepositoryError> {
    TargetRef::new(KEY, kind, id).map_err(RepositoryError::from)
}

fn status(label: String) -> Vec<ExportState> {
    vec![ExportState {
        property: WATERING_STATUS,
        value: StateValue::Category(label),
        observed_at: Utc::now(),
    }]
}

#[async_trait::async_trait]
impl ExportCatalog for IrrigationExportCatalog {
    #[tracing::instrument(level = "debug", skip_all, fields(orgs = orgs.len()))]
    async fn snapshot(
        &self,
        orgs: &BTreeSet<Id<Organization>>,
    ) -> Result<Vec<ExportThing>, RepositoryError> {
        let ids: Vec<Uuid> = orgs.iter().map(|o| o.value()).collect();
        let trees = sqlx::query!(
            r#"SELECT id, number, species, planting_year, latitude, longitude, tree_cluster_id,
                      description, organization_id, watering_status::text AS "watering_status!"
               FROM trees WHERE organization_id = ANY($1)"#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;
        let clusters = sqlx::query!(
            r#"SELECT id, name, address, description, latitude, longitude, archived, organization_id,
                      soil_condition::text AS "soil_condition!", watering_status::text AS "watering_status!"
               FROM tree_clusters WHERE organization_id = ANY($1)"#,
            &ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut things = Vec::with_capacity(trees.len() + clusters.len());
        for t in trees {
            things.push(ExportThing {
                target: target(TREE_KIND, t.id)?,
                name: format!("Baum {}", t.number),
                description: t.description.clone().unwrap_or_else(|| t.species.clone()),
                attributes: BTreeMap::from([
                    ("number".into(), json!(t.number)),
                    ("species".into(), json!(t.species)),
                    ("planting_year".into(), json!(t.planting_year)),
                    (
                        "cluster_id".into(),
                        json!(t.tree_cluster_id.map(|c| c.to_string())),
                    ),
                    (
                        "organization_id".into(),
                        json!(t.organization_id.to_string()),
                    ),
                ]),
                coordinate: Coordinate::new(t.latitude, t.longitude).ok(),
                states: status(t.watering_status),
            });
        }
        for c in clusters {
            things.push(ExportThing {
                target: target(CLUSTER_KIND, c.id)?,
                name: c.name,
                description: c.description,
                attributes: BTreeMap::from([
                    ("address".into(), json!(c.address)),
                    ("soil_condition".into(), json!(c.soil_condition)),
                    ("archived".into(), json!(c.archived)),
                    (
                        "organization_id".into(),
                        json!(c.organization_id.to_string()),
                    ),
                ]),
                coordinate: c
                    .latitude
                    .zip(c.longitude)
                    .and_then(|(lat, lon)| Coordinate::new(lat, lon).ok()),
                states: status(c.watering_status),
            });
        }
        Ok(things)
    }
}
