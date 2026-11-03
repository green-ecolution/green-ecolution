use std::collections::BTreeSet;

use domain::{
    Id,
    data_export::{ExportCatalog, StateValue},
};
use irrigation_server::infra::export_catalog::IrrigationExportCatalog;
use uuid::Uuid;

use crate::{helpers::spawn_app, organizations::ROOT_ORG_ID};

#[tokio::test]
async fn irrigation_exports_trees_and_clusters_of_the_given_organizations() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let cluster = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tree_clusters (id, name, address, description, moisture_level, soil_condition, watering_status, archived, latitude, longitude, organization_id)
         VALUES ($1, 'Süd', 'Hauptstr.', 'Beet', 0, 'unknown', 'bad', false, 54.79, 9.43, $2)",
    )
    .bind(cluster)
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();
    let tree = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO trees (id, tree_cluster_id, number, species, planting_year, latitude, longitude, watering_status, organization_id)
         VALUES ($1, $2, '12', 'Linde', 2010, 54.791, 9.431, 'good', $3)",
    )
    .bind(tree)
    .bind(cluster)
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();

    let catalog = IrrigationExportCatalog::new(app.db_pool.clone());
    let things = catalog
        .snapshot(&BTreeSet::from([Id::new(root)]))
        .await
        .unwrap();

    let tree_thing = things
        .iter()
        .find(|t| t.target.id == tree)
        .expect("tree exported");
    assert_eq!(tree_thing.target.kind.as_str(), "tree");
    assert_eq!(tree_thing.name, "Baum 12");
    assert_eq!(tree_thing.attributes["cluster_id"], cluster.to_string());
    assert_eq!(
        tree_thing.states[0].value,
        StateValue::Category("good".into())
    );
    let cluster_thing = things
        .iter()
        .find(|t| t.target.id == cluster)
        .expect("cluster exported");
    assert_eq!(
        cluster_thing.states[0].value,
        StateValue::Category("bad".into())
    );

    let other = catalog
        .snapshot(&BTreeSet::from([Id::new(Uuid::now_v7())]))
        .await
        .unwrap();
    assert!(
        other
            .iter()
            .all(|t| t.target.id != tree && t.target.id != cluster)
    );
}

#[tokio::test]
async fn waste_exports_bins_with_fill_level_and_state() {
    let app = spawn_app().await;
    let root = Uuid::parse_str(ROOT_ORG_ID).unwrap();
    let bin = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO waste_bins (id, number, latitude, longitude, geometry, volume_liters, depth_cm, organization_id, fill_fraction, fill_measured_at)
         VALUES ($1, 'B-1', 54.79, 9.43, ST_SetSRID(ST_MakePoint(9.43, 54.79), 4326), 240, 100, $2, 0.9, now())",
    )
    .bind(bin)
    .bind(root)
    .execute(&app.db_pool)
    .await
    .unwrap();

    let catalog = app
        .state
        .export_registry
        .catalogs()
        .into_iter()
        .find(|(key, _)| key.as_str() == "waste")
        .map(|(_, catalog)| catalog)
        .expect("waste registers an export catalog");
    let things = catalog
        .snapshot(&BTreeSet::from([Id::new(root)]))
        .await
        .unwrap();

    let thing = things
        .iter()
        .find(|t| t.target.id == bin)
        .expect("bin exported");
    assert_eq!(thing.name, "Behälter B-1");
    let level = thing
        .states
        .iter()
        .find(|s| s.property.key == "fill_level")
        .unwrap();
    assert_eq!(level.value, StateValue::Measurement(90.0));
    let state = thing
        .states
        .iter()
        .find(|s| s.property.key == "fill_state")
        .unwrap();
    assert_eq!(state.value, StateValue::Category("full".into()));
}
