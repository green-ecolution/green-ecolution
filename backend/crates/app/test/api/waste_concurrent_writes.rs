use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use domain::{
    Id, RepositoryError,
    authorization::Visibility,
    events::{DomainEvent, SensorDataReceivedPayload, SensorReadings},
    module::ModuleKey,
    planning::{StopOutcome, StopRef},
    sensor::{SensorId, data::DistanceReading},
    shared::pagination::{Pagination, SearchPage},
    target::TargetRef,
};
use serde_json::json;
use server::service::event_bus::EventHandler;
use sqlx::PgPool;
use uuid::Uuid;
use waste_domain::waste_bin::{
    WasteBin, WasteBinReader, WasteBinSearchQuery, WasteBinView, WasteBinWriter,
};
use waste_server::{
    handlers::{emptied::BinsEmptied, fill_level::FillLevelFromSensor},
    infra::pg_waste_bin::PgWasteBinRepository,
};

use crate::{
    helpers::TestApp,
    organizations::ROOT_ORG_ID,
    settings_repo::child_org,
    waste_bins::{create_bin, id_of, waste_app},
};

/// Hands out the bin as it was before a concurrent write, like a handler that
/// loaded it just before an HTTP request saved.
struct Stale(WasteBin);

#[async_trait::async_trait]
impl WasteBinReader for Stale {
    async fn by_id(&self, _id: Id<WasteBin>) -> Result<WasteBin, RepositoryError> {
        Ok(self.0.clone())
    }
    async fn by_ids(&self, _ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError> {
        unimplemented!()
    }
    async fn view_by_id(&self, _id: Id<WasteBin>) -> Result<WasteBinView, RepositoryError> {
        unimplemented!()
    }
    async fn view_by_ids(
        &self,
        _ids: &[Id<WasteBin>],
    ) -> Result<Vec<WasteBinView>, RepositoryError> {
        unimplemented!()
    }
    async fn view_search(
        &self,
        _query: WasteBinSearchQuery,
        _pagination: Pagination,
        _visibility: &Visibility,
    ) -> Result<SearchPage<WasteBinView>, RepositoryError> {
        unimplemented!()
    }
}

struct Row {
    organization_id: Uuid,
    number: String,
    fill_fraction: Option<f64>,
    fill_measured_at: Option<DateTime<Utc>>,
    last_emptied_at: Option<DateTime<Utc>>,
}

async fn row(pool: &PgPool, id: Uuid) -> Row {
    let (organization_id, number, fill_fraction, fill_measured_at, last_emptied_at) =
        sqlx::query_as(
            "SELECT organization_id, number, fill_fraction, fill_measured_at, last_emptied_at
             FROM waste_bins WHERE id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    Row {
        organization_id,
        number,
        fill_fraction,
        fill_measured_at,
        last_emptied_at,
    }
}

async fn transfer(app: &TestApp, bin: Uuid, to: Uuid) {
    let res = app
        .patch_json(
            &format!("/api/v1/waste-bins/{bin}/organization"),
            &json!({ "organization_id": to }),
        )
        .await;
    assert_eq!(res.status(), 204, "{}", res.text().await.unwrap());
}

fn reading(bin: Uuid, centimeters: f64) -> DomainEvent {
    DomainEvent::SensorDataReceived(SensorDataReceivedPayload {
        sensor_id: SensorId::new("eui-concurrent").unwrap(),
        target: Some(TargetRef::new("waste", "bin", bin).unwrap()),
        readings: SensorReadings::Distances(vec![DistanceReading {
            depth_cm: 0,
            centimeters,
        }]),
    })
}

fn finished(bin: Uuid, at: DateTime<Utc>) -> DomainEvent {
    DomainEvent::OperationPlanFinished {
        plan_id: Id::new_v7(),
        module: ModuleKey::new("waste").unwrap(),
        stops: vec![],
        finished_at: at,
        outcomes: vec![StopOutcome {
            stop: StopRef::new(bin),
            completed: true,
            amount: None,
        }],
    }
}

fn hour(h: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 2, h, 0, 0).unwrap()
}

#[tokio::test]
async fn a_reading_does_not_undo_a_concurrent_transfer() {
    let app = waste_app().await;
    let child = child_org(&app, "Bauhof", ROOT_ORG_ID.parse().unwrap()).await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let repo = Arc::new(PgWasteBinRepository::new(app.db_pool.clone()));
    let before_transfer = repo.by_id(Id::new(bin)).await.unwrap();

    transfer(&app, bin, child).await;
    FillLevelFromSensor::new(Arc::new(Stale(before_transfer)), repo)
        .handle(&reading(bin, 30.0))
        .await
        .unwrap();

    let row = row(&app.db_pool, bin).await;
    assert_eq!(row.organization_id, child);
    assert_eq!(row.number, "AB-1");
    assert_eq!(row.fill_fraction, Some(0.75));
}

#[tokio::test]
async fn an_emptying_does_not_undo_a_transfer_or_a_newer_emptying() {
    let app = waste_app().await;
    let child = child_org(&app, "Bauhof", ROOT_ORG_ID.parse().unwrap()).await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let repo = Arc::new(PgWasteBinRepository::new(app.db_pool.clone()));
    let before_emptying = repo.by_id(Id::new(bin)).await.unwrap();

    BinsEmptied::new(repo.clone(), repo.clone())
        .handle(&finished(bin, hour(12)))
        .await
        .unwrap();
    transfer(&app, bin, child).await;
    BinsEmptied::new(Arc::new(Stale(before_emptying)), repo)
        .handle(&finished(bin, hour(10)))
        .await
        .unwrap();

    let row = row(&app.db_pool, bin).await;
    assert_eq!(row.organization_id, child);
    assert_eq!(row.last_emptied_at, Some(hour(12)));
}

#[tokio::test]
async fn an_older_reading_does_not_overwrite_a_newer_one() {
    let app = waste_app().await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let repo = PgWasteBinRepository::new(app.db_pool.clone());
    let mut older = repo.by_id(Id::new(bin)).await.unwrap();
    let mut newer = older.clone();

    assert!(newer.record_fill_level(30.0, hour(12)));
    repo.save_fill_level(&newer).await.unwrap();
    assert!(older.record_fill_level(90.0, hour(10)));
    repo.save_fill_level(&older).await.unwrap();

    let row = row(&app.db_pool, bin).await;
    assert_eq!(row.fill_fraction, Some(0.75));
    assert_eq!(row.fill_measured_at, Some(hour(12)));
}

#[tokio::test]
async fn an_older_emptying_does_not_overwrite_a_newer_one() {
    let app = waste_app().await;
    let bin = id_of(&create_bin(&app, "AB-1").await);
    let repo = PgWasteBinRepository::new(app.db_pool.clone());
    let mut older = repo.by_id(Id::new(bin)).await.unwrap();
    let mut newer = older.clone();

    newer.mark_emptied(hour(12));
    repo.save_emptied(&newer).await.unwrap();
    older.mark_emptied(hour(10));
    repo.save_emptied(&older).await.unwrap();

    assert_eq!(row(&app.db_pool, bin).await.last_emptied_at, Some(hour(12)));
}
