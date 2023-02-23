use std::net::TcpListener;

use axum::BoxError;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    run(listener)?.await?;

    Ok(())
}
