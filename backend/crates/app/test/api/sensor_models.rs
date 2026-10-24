use crate::helpers::spawn_app;

#[tokio::test]
async fn list_sensor_models_returns_seeded_models() {
    let app = spawn_app().await;
    let r = app.get("/api/v1/sensors/models").await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    let arr = body.as_array().expect("models response is an array");
    let names: Vec<&str> = arr.iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"EcoDrizzler"), "names = {names:?}");
    assert!(names.contains(&"GES-1000"), "names = {names:?}");
}

#[tokio::test]
async fn get_sensor_model_returns_abilities_for_eco_drizzler() {
    let app = spawn_app().await;
    let model_id = app.ecodrizzler_model_id().await;
    let r = app
        .get(&format!("/api/v1/sensors/models/{}", model_id))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["name"], "EcoDrizzler");
    let abilities = body["abilities"].as_array().unwrap();
    // 3x soil_tension (30/60/90) + 1x soil_moisture@15 + temperature@15 + humidity@15
    assert_eq!(abilities.len(), 6, "abilities = {abilities:#?}");

    let st_count = abilities
        .iter()
        .filter(|a| a["ability"] == "soil_tension")
        .count();
    assert_eq!(st_count, 3, "abilities = {abilities:#?}");
}

#[tokio::test]
async fn get_sensor_model_returns_abilities_for_ges_1000() {
    let app = spawn_app().await;
    let model_id = app.ges_1000_model_id().await;
    let r = app
        .get(&format!("/api/v1/sensors/models/{}", model_id))
        .await;
    assert_eq!(r.status().as_u16(), 200);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["name"], "GES-1000");
    let abilities = body["abilities"].as_array().unwrap();
    assert_eq!(abilities.len(), 5, "abilities = {abilities:#?}");
    let depths_for = |name: &str| {
        let mut ds: Vec<i64> = abilities
            .iter()
            .filter(|a| a["ability"] == name)
            .filter_map(|a| a["depth_cm"].as_i64())
            .collect();
        ds.sort_unstable();
        ds
    };
    assert_eq!(depths_for("soil_moisture"), vec![40, 80]);
    assert_eq!(depths_for("temperature"), vec![40, 80]);
    assert_eq!(depths_for("battery"), vec![0]);
}

#[tokio::test]
async fn get_unknown_sensor_model_returns_404() {
    let app = spawn_app().await;
    let r = app
        .get(&format!("/api/v1/sensors/models/{}", uuid::Uuid::now_v7()))
        .await;
    assert_eq!(r.status().as_u16(), 404);
}
