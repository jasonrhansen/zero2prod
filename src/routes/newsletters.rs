use anyhow::Context;
use axum::{extract::State, response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::warn;

use crate::{app_state::AppState, domain::SubscriberEmail, email_client::EmailClient};

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status_code.into_response()
    }
}

pub async fn publish_newsletter<E>(
    State(state): State<AppState<E>>,
    body: Json<BodyData>,
) -> Result<StatusCode, PublishError>
where
    E: EmailClient + Clone,
{
    let subscribers = get_confirmed_subscribers(&state.db_pool).await?;
    for subscriber in subscribers {
        state
            .email_client
            .send_email(&subscriber.email, &body.title, &body.content)
            .await
            .with_context(|| format!("Failed to send newsletter issues to {}", subscriber.email))?;
    }

    Ok(StatusCode::OK)
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
pub async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .filter_map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Some(ConfirmedSubscriber { email }),
            Err(error) => {
                warn!(
                    "A confirmed subscriberis using an invalid email address.\n{}.",
                    error
                );
                None
            }
        })
        .collect();

    Ok(confirmed_subscribers)
}
