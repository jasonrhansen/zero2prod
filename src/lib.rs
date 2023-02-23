use axum::response::IntoResponse;
use axum::routing::IntoMakeService;
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;
use hyper::StatusCode;
use std::net::SocketAddr;

pub fn run() -> Result<Server<AddrIncoming, IntoMakeService<Router>>, BoxError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let server = axum::Server::bind(&addr).serve(app().into_make_service());

    Ok(server)
}

fn app() -> Router {
    Router::new().route("/health_check", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
