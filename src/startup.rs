use axum::error_handling::HandleErrorLayer;
use axum::routing::{post, IntoMakeService};
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;
use hyper::{Method, StatusCode, Uri};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

use std::net::TcpListener;
use std::time::Duration;

use crate::app_state::AppState;
use crate::routes;

fn app(shared_state: AppState) -> Router {
    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        // Return an error after 30 seconds
        .timeout(Duration::from_secs(30))
        // Shed load if we're receiving too many requests
        .load_shed()
        // Process at most 100 requests concurrently
        .concurrency_limit(100)
        // Tracing
        .layer(TraceLayer::new_for_http())
        // Compress response bodies
        .layer(CompressionLayer::new());

    Router::new()
        .route("/health_check", get(routes::health_check))
        .route(
            "/subscriptions",
            post(routes::subscribe).with_state(shared_state),
        )
        .layer(middleware_stack)
}

pub fn run(
    listener: TcpListener,
    shared_state: AppState,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, BoxError> {
    let server = axum::Server::from_tcp(listener)?.serve(app(shared_state).into_make_service());

    Ok(server)
}

async fn handle_error(method: Method, uri: Uri, err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("`{} {}` failed with {}", method, uri, err),
        )
    }
}
