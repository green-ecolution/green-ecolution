use serde_json::Value;
use server::http::openapi_doc;

use crate::helpers::spawn_app;

fn dumped_spec() -> Value {
    let module_docs = app::modules::registered()
        .iter()
        .map(|module| module.openapi())
        .collect();
    serde_json::to_value(openapi_doc("http://127.0.0.1:3030", module_docs))
        .expect("the OpenAPI document serializes")
}

async fn served_spec() -> Value {
    let app = spawn_app().await;
    let response = app.get("/api-docs/openapi.json").await;
    assert_eq!(response.status().as_u16(), 200);
    response.json().await.expect("served spec is JSON")
}

#[tokio::test]
async fn the_served_spec_carries_the_same_tags_as_the_dumped_one() {
    let (served, dumped) = (served_spec().await, dumped_spec());

    assert_eq!(served["tags"], dumped["tags"]);
    assert!(
        served["tags"]
            .as_array()
            .expect("tags is an array")
            .iter()
            .any(|tag| tag["name"] == "Trees" && tag["description"].is_string()),
        "module tags with their descriptions are served: {}",
        served["tags"]
    );
}

#[tokio::test]
async fn the_served_spec_has_the_same_paths_and_schemas_as_the_dumped_one() {
    let (served, dumped) = (served_spec().await, dumped_spec());

    assert_eq!(served["paths"], dumped["paths"]);
    assert_eq!(served["components"], dumped["components"]);
}
