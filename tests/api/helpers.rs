use axum::async_trait;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    domain::SubscriberEmail,
    email_client::EmailClient,
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
    pub connection_pool: PgPool,
}

#[derive(Clone)]
pub struct TestEmailClient {}

#[async_trait]
impl EmailClient for TestEmailClient {
    async fn send_email(
        &self,
        _recipient: SubscriberEmail,
        _subject: &str,
        _html_content: &str,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read configuration.");
    config.application.host = "127.0.0.1".to_string();
    config.application.port = 0;
    config.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&config.database).await;

    let email_client = TestEmailClient {};

    let application = Application::build(config.clone(), email_client)
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        connection_pool,
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
