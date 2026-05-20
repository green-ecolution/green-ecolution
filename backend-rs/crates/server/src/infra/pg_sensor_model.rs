use std::str::FromStr;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::{
    Id, RepositoryError,
    sensor_model::{
        SensorAbility, SensorAbilityName, SensorAbilityUnit, SensorModel, SensorModelAbility,
        SensorModelName, SensorModelReader,
    },
};

pub struct PgSensorModelRepository {
    pool: PgPool,
}

impl PgSensorModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SensorModelReader for PgSensorModelRepository {
    #[tracing::instrument(level = "trace", skip_all)]
    async fn list(&self) -> Result<Vec<SensorModel>, RepositoryError> {
        let rows = sqlx::query!(r#"SELECT id, name, description FROM sensor_models ORDER BY id"#)
            .fetch_all(&self.pool)
            .await?;

        let mut out = Vec::with_capacity(rows.len());
        for m in rows {
            let abilities = load_abilities(&self.pool, m.id).await?;
            out.push(build_model(m.id, m.name, m.description, abilities));
        }
        Ok(out)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_id(&self, id: Id<SensorModel>) -> Result<SensorModel, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, name, description FROM sensor_models WHERE id = $1"#,
            id.value()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let abilities = load_abilities(&self.pool, row.id).await?;
        Ok(build_model(row.id, row.name, row.description, abilities))
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn by_name(&self, name: &SensorModelName) -> Result<SensorModel, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id, name, description FROM sensor_models WHERE name = $1"#,
            name.as_str()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RepositoryError::NotFound)?;

        let abilities = load_abilities(&self.pool, row.id).await?;
        Ok(build_model(row.id, row.name, row.description, abilities))
    }
}

fn build_model(
    id: Uuid,
    name: String,
    description: Option<String>,
    abilities: Vec<SensorModelAbility>,
) -> SensorModel {
    SensorModel {
        id: Id::new(id),
        name: SensorModelName::reconstitute(name),
        description,
        abilities,
    }
}

async fn load_abilities(
    pool: &PgPool,
    model_id: Uuid,
) -> Result<Vec<SensorModelAbility>, RepositoryError> {
    let rows = sqlx::query!(
        r#"SELECT sma.id           AS mapping_id,
                  sma.depth_cm,
                  sa.id             AS ability_id,
                  sa.ability        AS ability_name,
                  sa.unit           AS "unit: SensorAbilityUnit"
           FROM sensor_model_abilities sma
           JOIN sensor_abilities       sa ON sa.id = sma.sensor_ability_id
           WHERE sma.sensor_model_id = $1
           ORDER BY sa.id, sma.depth_cm"#,
        model_id
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let name = SensorAbilityName::from_str(&r.ability_name).map_err(|e| {
                RepositoryError::DataIntegrity(format!("unknown ability in DB: {e}"))
            })?;
            Ok(SensorModelAbility {
                id: r.mapping_id,
                ability: SensorAbility {
                    id: r.ability_id,
                    name,
                    unit: r.unit,
                },
                depth_cm: r.depth_cm,
            })
        })
        .collect()
}
