use std::fmt::Display;

use axum::async_trait;
use lettre::{
    error::Error as EmailError, message::header::ContentType, transport::smtp::Error as SmtpError,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::domain::SubscriberEmail;

#[derive(Debug)]
pub enum SendEmailError {
    EmailError(EmailError),
    SmtpError(SmtpError),
}

impl Display for SendEmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send email")
    }
}

impl std::error::Error for SendEmailError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SendEmailError::EmailError(e) => Some(e),
            SendEmailError::SmtpError(e) => Some(e),
        }
    }
}

impl From<EmailError> for SendEmailError {
    fn from(value: EmailError) -> Self {
        SendEmailError::EmailError(value)
    }
}

impl From<SmtpError> for SendEmailError {
    fn from(value: SmtpError) -> Self {
        SendEmailError::SmtpError(value)
    }
}

#[async_trait]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), SendEmailError>;
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
