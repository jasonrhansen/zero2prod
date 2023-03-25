use axum::error_handling::HandleErrorLayer;
use axum::middleware::from_fn;
use axum::routing::{post, IntoMakeService};
use axum::{routing::get, Router};
use axum::{BoxError, Server};
use axum_flash::Key;
use axum_sessions::async_session::SessionStore;
use axum_sessions::SessionLayer;
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
use crate::authentication::reject_anonymous_users;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes;

pub struct Application {
    port: u16,
    server: Server<AddrIncoming, IntoMakeService<Router>>,
}

impl Application {
    pub async fn build<E, S>(
        config: Settings,
        email_client: E,
        session_store: S,
    ) -> Result<Self, anyhow::Error>
    where
        E: EmailClient + Clone + Send + Sync + 'static,
        S: SessionStore,
    {
        // TODO: get secret key from environment
        let secret_key = Key::generate();

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
                flash_config: axum_flash::Config::new(secret_key.clone()),
            },
            session_store,
            secret_key,
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

fn app<E, S>(shared_state: AppState<E>, session_store: S, secret_key: Key) -> Router
where
    E: EmailClient + Clone + Send + Sync + 'static,
    S: SessionStore,
{
    let session_layer = SessionLayer::new(session_store, secret_key.master());

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        // Return an error after 30 seconds
        .timeout(Duration::from_secs(30))
        // Shed load if we're receiving too many requests
        .load_shed()
        // Process at most 100 requests concurrently
        .concurrency_limit(100)
        .layer(session_layer)
        // Set request id before the tracing layer so it ends up in the logs.
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

    let admin_routes = Router::new()
        .route("/dashboard", get(routes::admin_dashboard))
        .route(
            "/password",
            get(routes::change_password_form).post(routes::change_password),
        )
        .route("/logout", post(routes::log_out))
        .layer(from_fn(reject_anonymous_users));

    let with_state = Router::new()
        .route("/", get(routes::home))
        .route("/login", get(routes::login_form).post(routes::login))
        .route("/subscriptions", post(routes::subscribe))
        .route("/subscriptions/confirm", get(routes::confirm))
        .route("/newsletters", post(routes::publish_newsletter))
        .nest("/admin", admin_routes)
        .with_state(shared_state);

    Router::new()
        .route("/health_check", get(routes::health_check))
        .merge(with_state)
        .layer(middleware_stack)
}

pub fn run<E, S>(
    listener: TcpListener,
    shared_state: AppState<E>,
    session_store: S,
    secret_key: Key,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>, anyhow::Error>
where
    E: EmailClient + Clone + Send + Sync + 'static,
    S: SessionStore,
{
    let server = axum::Server::from_tcp(listener)?
        .serve(app(shared_state, session_store, secret_key).into_make_service());

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
