use axum::{debug_handler, extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
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
        println!("Failed to execute query: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
