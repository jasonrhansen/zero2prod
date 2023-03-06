use std::sync::{Arc, Mutex};

use axum::async_trait;
use hyper::StatusCode;
use linkify::Link;
use once_cell::sync::Lazy;
use reqwest::Url;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    domain::SubscriberEmail,
    email_client::{self, EmailClient},
    startup::Application,
    telemetry,
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: Arc<Mutex<TestEmailServer>>,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn confirmation_link(&self) -> Url {
        let email_server = self.email_server.lock().unwrap();
        let mut confirmation_link =
            Url::parse(get_links(&email_server.sends[0].html_content)[0].as_str()).unwrap();

        assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

        // Rewrite URL to use test port.
        confirmation_link.set_port(Some(self.port)).unwrap();

        confirmation_link
    }
}

#[derive(Clone, Default)]
pub struct TestEmailClient {
    inner: Arc<Mutex<TestEmailServer>>,
}

#[async_trait]
impl EmailClient for TestEmailClient {
    async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), email_client::SendEmailError> {
        self.inner.lock().unwrap().sends.push(TestEmail {
            recipient,
            subject: subject.to_owned(),
            html_content: html_content.to_owned(),
        });

        Ok(())
    }
}

#[derive(Clone)]
pub struct TestEmail {
    pub recipient: SubscriberEmail,
    pub subject: String,
    pub html_content: String,
}

#[derive(Clone, Default)]
pub struct TestEmailServer {
    pub sends: Vec<TestEmail>,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.application.host = "127.0.0.1".to_string();
    config.application.port = 0;
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&config.database).await;

    let email_client = TestEmailClient::default();
    let email_client_inner = Arc::clone(&email_client.inner);

    let application = Application::build(config.clone(), email_client)
        .await
        .expect("Failed to build application");
    let application_port = application.port();

    let address = format!("http://127.0.0.1:{}", application.port());
    drop(tokio::spawn(application.run_until_stopped()));

    TestApp {
        address,
        port: application_port,
        db_pool: connection_pool,
        email_server: email_client_inner,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create test database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn get_links(html_text: &str) -> Vec<Link> {
    linkify::LinkFinder::new()
        .links(html_text)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect()
}

pub fn assert_status_code(expected: StatusCode, actual: StatusCode, payload: &str) {
    assert_eq!(
        expected, actual,
        "The API did not return a {} when the payload was {}.",
        expected, payload
    );
}
