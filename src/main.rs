use axum::response::IntoResponse;
use axum::{routing::get, Router};
use hyper::StatusCode;
use std::error::Error;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await?;

    Ok(())
}

fn app() -> Router {
    Router::new().route("/health_check", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
