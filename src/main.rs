use std::net::TcpListener;

use axum::BoxError;
use sqlx::PgPool;
use zero2prod::{app_state::AppState, configuration::get_configuration, logger, startup::run};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let config = get_configuration().expect("Failed to read configuration.");

    logger::initialize();

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, AppState { connection_pool })?.await?;

    Ok(())
}
