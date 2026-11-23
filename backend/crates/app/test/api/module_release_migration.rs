use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    helpers::{BEFORE_MODULE_RELEASE, TestApp, spawn_app_on_migrated_until},
    organizations::ROOT_ORG_ID,
};

const A: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa01);
const B: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa02);
const C: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa03);
const D: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa04);
const E: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa05);
const F: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa06);
const A1: Uuid = Uuid::from_u128(0x0198_0000_0000_7000_8000_0000_0000_aa11);

async fn seed_old_schema(pool: &PgPool) {
    let root: Uuid = ROOT_ORG_ID.parse().unwrap();
    for (id, parent, name) in [
        (A, root, "A"),
        (B, root, "B"),
        (C, root, "C"),
        (D, root, "D"),
        (E, root, "E"),
        (F, root, "F"),
        (A1, A, "A1"),
    ] {
        sqlx::query("INSERT INTO organizations (id, parent_id, name) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(parent)
            .bind(name)
            .execute(pool)
            .await
            .unwrap();
    }
    for (org, key, enabled, set_by) in [
        (root, "waste", true, root),
        (B, "waste", false, B),
        (C, "waste", false, root),
        (D, "irrigation", false, D),
        (E, "irrigation", true, root),
        (F, "other_module", false, F),
        (A1, "waste", false, A1),
    ] {
        sqlx::query(
            "INSERT INTO organization_modules (organization_id, module_key, enabled, set_by)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(org)
        .bind(key)
        .bind(enabled)
        .bind(set_by)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn get_json(app: &TestApp, path: &str) -> Value {
    let resp = app.get(path).await;
    assert_eq!(resp.status(), 200, "GET {path}");
    resp.json().await.unwrap()
}

fn state(releases: &Value, tenant: Uuid, key: &str) -> (bool, bool) {
    let entry = releases["tenants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["organization_id"] == tenant.to_string())
        .expect("tenant is listed");
    let module = entry["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == key)
        .unwrap();
    (
        module["released"].as_bool().unwrap(),
        module["enabled"].as_bool().unwrap(),
    )
}

#[tokio::test]
async fn the_old_switches_become_releases_per_tenant() {
    let app = spawn_app_on_migrated_until(BEFORE_MODULE_RELEASE, async |pool: &PgPool| {
        seed_old_schema(pool).await
    })
    .await;

    let releases = get_json(&app, "/api/v1/modules/releases").await;
    assert_eq!(state(&releases, A, "irrigation"), (true, true));
    assert_eq!(state(&releases, A, "waste"), (true, true));
    assert_eq!(state(&releases, B, "waste"), (true, false));
    assert!(!state(&releases, C, "waste").0);
    assert_eq!(state(&releases, D, "irrigation"), (true, false));
    assert_eq!(state(&releases, E, "irrigation"), (true, true));

    let other: Option<bool> = sqlx::query_scalar(
        "SELECT enabled FROM organization_modules
         WHERE organization_id = $1 AND module_key = 'other_module'",
    )
    .bind(F)
    .fetch_optional(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(other, Some(false));

    let sub_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM organization_modules WHERE organization_id = $1")
            .bind(A1)
            .fetch_one(&app.db_pool)
            .await
            .unwrap();
    assert_eq!(sub_rows, 0);

    let sub = get_json(&app, &format!("/api/v1/organizations/{A1}/modules")).await;
    let waste = sub["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["key"] == "waste")
        .unwrap();
    assert_eq!(waste["enabled"], true);
    assert_eq!(waste["switchable"], false);

    let root: (bool, bool) = sqlx::query_as(
        "SELECT released, enabled FROM organization_modules
         WHERE organization_id = $1 AND module_key = 'waste'",
    )
    .bind(Uuid::parse_str(ROOT_ORG_ID).unwrap())
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(root, (true, true));
}
