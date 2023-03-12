use axum::error_handling::HandleErrorLayer;
use axum::routing::{post, IntoMakeService};
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use hyper::server::conn::AddrIncoming;
use hyper::{Method, StatusCode, Uri};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::request_id::MakeRequestUuid;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use tracing::Level;

use std::net::TcpListener;
use std::time::Duration;

use crate::app_state::AppState;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes;

pub struct Application {
    port: u16,
    server: Server<AddrIncoming, IntoMakeService<Router>>,
}

impl Application {
    pub async fn build<E>(config: Settings, email_client: E) -> Result<Self, anyhow::Error>
    where
        E: EmailClient + Clone + Send + Sync + 'static,
    {
        let db_pool = get_connection_pool(&config.database);
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            AppState {
                db_pool,
                email_client,
                base_url: config.application.base_url,
            },
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), anyhow::Error> {
        self.server.await?;

        Ok(())
    }
}

pub fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}

fn app<E>(shared_state: AppState<E>) -> Router
where
    E: EmailClient + Clone + Send + Sync + 'static,
{
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

    let with_state = Router::new()
        .route("/subscriptions", post(routes::subscribe))
        .route("/subscriptions/confirm", get(routes::confirm))
        .route("/newsletters", post(routes::publish_newsletter))
        .with_state(shared_state);

    Router::new()
        .route("/health_check", get(routes::health_check))
        .merge(with_state)
        .layer(middleware_stack)
}

pub fn run<E>(
    listener: TcpListener,
    shared_state: AppState<E>,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, anyhow::Error>
where
    E: EmailClient + Clone + Send + Sync + 'static,
{
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
