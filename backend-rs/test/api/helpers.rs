use green_ecolution::startup::Application;
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    _container: ContainerAsync<GenericImage>,
}

impl TestApp {
    pub async fn get(&self, path: &str) -> reqwest::Response {
        reqwest::Client::new()
            .get(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn post_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}{}", self.address, path))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn put_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .put(format!("{}{}", self.address, path))
            .json(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub async fn delete(&self, path: &str) -> reqwest::Response {
        reqwest::Client::new()
            .delete(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("failed to execute request")
    }
}

pub async fn spawn_app() -> TestApp {
    let image = GenericImage::new("postgis/postgis", "17-3.5")
        .with_exposed_port(5432.into())
        .with_wait_for(testcontainers::core::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres");

    let container: ContainerAsync<GenericImage> = image
        .start()
        .await
        .expect("failed to start postgis container");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("failed to get postgres port");

    let connection_string =
        format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres");

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("failed to run migrations");

    let app = Application::build_with_pool(db_pool.clone(), "127.0.0.1:0")
        .await
        .expect("failed to build application");
    let port = app.port();
    tokio::spawn(app.run_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
        port,
        db_pool,
        _container: container,
    }
}
