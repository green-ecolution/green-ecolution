use chrono::{Duration, Utc};
use domain::Id;
use domain::plugin::{Plugin, PluginSlug};
use server::infra::pg_plugin_proxy_session::PgProxySessionStore;
use server::service::plugin_proxy_service::{NewProxySession, ProxySessionStore};
use uuid::Uuid;

use crate::helpers::{TestApp, install_plugin, spawn_app_with_plugins};

async fn plugin_id(app: &TestApp, slug: &str) -> Id<Plugin> {
    app.state
        .plugin_reader
        .by_slug(&PluginSlug::new(slug).unwrap())
        .await
        .unwrap()
        .id
}

fn session(plugin: Id<Plugin>, hash: &str, minutes: i64) -> NewProxySession {
    NewProxySession {
        token_hash: hash.into(),
        plugin_id: plugin,
        user_id: Uuid::nil(),
        user_display_name: "Toni Tester".into(),
        expires_at: Utc::now() + Duration::minutes(minutes),
    }
}

#[tokio::test]
async fn a_live_session_is_found_and_an_expired_one_is_not() {
    let app = spawn_app_with_plugins().await;
    install_plugin(&app, "acme", &[]).await;
    let plugin = plugin_id(&app, "acme").await;
    let store = PgProxySessionStore::new(app.db_pool.clone());

    store.insert(session(plugin, "live", 10)).await.unwrap();
    store.insert(session(plugin, "stale", -1)).await.unwrap();

    let found = store.find("live").await.unwrap().expect("live session");
    assert_eq!(found.plugin_id, plugin);
    assert_eq!(found.user_display_name, "Toni Tester");
    assert!(store.find("stale").await.unwrap().is_none());
}

#[tokio::test]
async fn purge_removes_only_expired_sessions() {
    let app = spawn_app_with_plugins().await;
    install_plugin(&app, "acme", &[]).await;
    let plugin = plugin_id(&app, "acme").await;
    let store = PgProxySessionStore::new(app.db_pool.clone());
    store.insert(session(plugin, "live", 10)).await.unwrap();
    store.insert(session(plugin, "stale", -1)).await.unwrap();

    store.purge_expired(plugin).await.unwrap();

    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM plugin_proxy_sessions")
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(remaining, 1);
}
