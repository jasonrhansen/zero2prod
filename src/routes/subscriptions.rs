use axum::{debug_handler, extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use tracing::{error, info};
use uuid::Uuid;

use crate::app_state::AppState;

#[derive(Deserialize)]
pub struct SubscriptionFormData {
    email: String,
    name: String,
}

#[debug_handler]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<SubscriptionFormData>,
) -> impl IntoResponse {
    info!(
        "Adding '{}' '{}' as a new subscriber",
        form.email, form.name
    );
    info!("Saving new subscriber details in the database");
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&state.connection_pool)
    .await;

    if let Err(e) = result {
        error!("Failed to execute query: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!("New subscriber details have been saved");
    StatusCode::OK
}
