use std::net::TcpListener;

use axum::BoxError;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{app_state::AppState, configuration::get_configuration, startup::run, telemetry};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let connection_pool =
        PgPool::connect_lazy(&config.database.connection_string().expose_secret())
            .expect("Failed to create Postgres connection pool");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, AppState { connection_pool })?.await?;

    Ok(())
}
