use std::{net::TcpListener, time::Duration};

use axum::BoxError;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{app_state::AppState, configuration::get_configuration, startup::run, telemetry};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, AppState { connection_pool })?.await?;

    Ok(())
}
