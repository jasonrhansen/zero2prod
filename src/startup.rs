use axum::routing::{post, IntoMakeService};
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;

use std::net::TcpListener;

use crate::app_state::AppState;
use crate::routes;

fn app(shared_state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(routes::health_check))
        .route(
            "/subscriptions",
            post(routes::subscribe).with_state(shared_state),
        )
}

pub fn run(
    listener: TcpListener,
    shared_state: AppState,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, BoxError> {
    let server = axum::Server::from_tcp(listener)?.serve(app(shared_state).into_make_service());

    Ok(server)
}
