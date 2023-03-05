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

    let application = Application::build(config, email_client).await.unwrap();
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
