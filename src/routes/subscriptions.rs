use axum::{debug_handler, extract::State, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
};

#[derive(Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[debug_handler]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<SubscriptionFormData>,
) -> Result<StatusCode, (StatusCode, String)> {
    let new_subscriber = NewSubscriber {
        email: SubscriberEmail::parse(form.email)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?,
        name: SubscriberName::parse(form.name)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?,
    };
    insert_subscriber(&state.connection_pool, &new_subscriber)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, new_subscriber)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map(|_| ())
}
