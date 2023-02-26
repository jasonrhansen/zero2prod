use axum::{debug_handler, extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::app_state::AppState;

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
) -> impl IntoResponse {
    insert_subscriber(&state.connection_pool, &form)
        .await
        .map(|_| StatusCode::OK)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &SubscriptionFormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })
}
