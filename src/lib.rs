use axum::response::IntoResponse;
use axum::routing::IntoMakeService;
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;
use hyper::StatusCode;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, BoxError> {
    let server = axum::Server::from_tcp(listener)?.serve(app().into_make_service());

    Ok(server)
}

fn app() -> Router {
    Router::new().route("/health_check", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
