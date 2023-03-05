use axum::{extract::State, Form};
use chrono::Utc;
use hyper::StatusCode;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    app_state::AppState,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

impl TryFrom<SubscriptionFormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: SubscriptionFormData) -> Result<Self, Self::Error> {
        Ok(Self {
            name: SubscriberName::parse(value.name)?,
            email: SubscriberEmail::parse(value.email)?,
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe<E>(
    State(state): State<AppState<E>>,
    Form(form): Form<SubscriptionFormData>,
) -> Result<StatusCode, AppError>
where
    E: EmailClient + Clone,
{
    let new_subscriber: NewSubscriber = match form.try_into() {
        Ok(s) => s,
        Err(_) => return Ok(StatusCode::UNPROCESSABLE_ENTITY),
    };

    let mut transaction = state.db_pool.begin().await?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber).await?;

    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;

    transaction.commit().await?;

    send_confirmation_email(
        state.email_client,
        new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, confirmation_token)
)]
pub async fn send_confirmation_email<E>(
    email_client: E,
    new_subscriber: NewSubscriber,
    base_url: &str,
    confirmation_token: &str,
) -> Result<(), anyhow::Error>
where
    E: EmailClient + Clone,
{
    let confirmation_link =
        format!("{base_url}/subscriptions/confirm?subscription_token={confirmation_token}");

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                r#"
                <h1>Welcome to our newsletter!</h1>
                Click <a href="{}">here</a> to confirm your subscription.""
                "#,
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map(|_| ())?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store subscription token in the databse",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    (0..25)
        .map(|_| char::from(rng.sample(Alphanumeric)))
        .collect()
}
