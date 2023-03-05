use std::{net::TcpListener, time::Duration};

use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    app_state::AppState,
    configuration::{get_configuration, Settings},
    domain::SubscriberEmail,
    email_client::SmtpEmailClient,
    startup::run,
    telemetry,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let email_client = setup_email_client(&config);

    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;
    run(
        listener,
        AppState {
            connection_pool,
            email_client,
        },
    )?
    .await?;

    Ok(())
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
