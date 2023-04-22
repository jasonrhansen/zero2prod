use std::sync::Arc;

use axum::async_trait;
use lettre::{
    error::Error as EmailError, message::header::ContentType, transport::smtp::Error as SmtpError,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::domain::SubscriberEmail;

#[derive(Debug, thiserror::Error)]
pub enum SendEmailError {
    #[error(transparent)]
    EmailError(#[from] EmailError),
    #[error(transparent)]
    SmtpError(#[from] SmtpError),
}

#[async_trait]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), SendEmailError>;
}

pub type DynEmailClient = Arc<dyn EmailClient + Send + Sync>;

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
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), SendEmailError> {
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
