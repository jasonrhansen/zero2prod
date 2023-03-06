use std::error::Error;

use axum::async_trait;
use lettre::{
    message::header::ContentType, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::domain::SubscriberEmail;

#[derive(Debug)]
pub struct SendEmailError(Box<dyn Error>);

impl<E> From<E> for SendEmailError
where
    E: Error + 'static,
{
    fn from(err: E) -> Self {
        Self(Box::new(err))
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
