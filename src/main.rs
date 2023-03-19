use async_fred_session::RedisSessionStore;
use fred::{pool::RedisPool, prelude::*};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;

use zero2prod::{
    configuration::{get_configuration, Settings},
    domain::SubscriberEmail,
    email_client::SmtpEmailClient,
    startup::Application,
    telemetry,
};

#[tokio::main]
async fn main() {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");
    let email_client = setup_email_client(&config);
    let session_store = setup_redis_session_store(&config).await;

    let application = Application::build(config, email_client, session_store)
        .await
        .unwrap();
    application.run_until_stopped().await.unwrap();
}

fn setup_email_client(config: &Settings) -> SmtpEmailClient {
    let smtp_creds = Credentials::new(
        config.email_client.smtp_username.clone(),
        config.email_client.smtp_password.expose_secret().clone(),
    );
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.email_client.smtp_relay)
        .expect("Unable to create SMTP transport")
        .credentials(smtp_creds)
        .build();
    let sender = SubscriberEmail::parse(config.email_client.sender_email.clone())
        .expect("Invalid sender email address");

    SmtpEmailClient::new(mailer, sender)
}

async fn setup_redis_session_store(config: &Settings) -> RedisSessionStore {
    let redis_config = RedisConfig::from_url(config.redis_uri.expose_secret().as_ref()).unwrap();
    let redis_pool = RedisPool::new(redis_config, 1).unwrap();
    redis_pool.connect(None);
    redis_pool.wait_for_connect().await.unwrap();

    RedisSessionStore::from_pool(redis_pool, Some("async-fred-session/".into()))
}
