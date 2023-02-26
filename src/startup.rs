use axum::error_handling::HandleErrorLayer;
use axum::routing::{post, IntoMakeService};
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;
use hyper::{Method, StatusCode, Uri};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::request_id::MakeRequestUuid;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use tracing::Level;

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
        // Set request id before the tra layer so it ends up in the logs.
        .set_x_request_id(MakeRequestUuid)
        // Tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        .propagate_x_request_id()
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
