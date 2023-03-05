use axum::async_trait;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::domain::SubscriberEmail;

#[async_trait]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Clone)]
pub struct SmtpEmailClient {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    sender: SubscriberEmail,
}

impl SmtpEmailClient {
    pub fn new(mailer: AsyncSmtpTransport<Tokio1Executor>, sender: SubscriberEmail) -> Self {
        Self { mailer, sender }
    }
}

#[async_trait]
impl EmailClient for SmtpEmailClient {
    async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), anyhow::Error> {
        let email = Message::builder()
            .from(self.sender.as_ref().parse().unwrap())
            .to(recipient.as_ref().parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_content.to_string())?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
