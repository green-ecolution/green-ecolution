use green_ecolution::{configuration::get_configuration, startup::Application};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt().init();

    let config = get_configuration().expect("failed to read configuration");
    let app = Application::build(config).await?;
    app.run_until_stopped().await
}
