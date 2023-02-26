use axum::{debug_handler, extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use tracing::{error, info, info_span, Instrument};
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
    let request_id = Uuid::new_v4();
    let subscribe_span = info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    let _subscribe_span_guard = subscribe_span.enter();

    let query_span = info_span!("Saving new subscriber details in the database");
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
    .instrument(query_span)
    .await;

    if let Err(e) = result {
        error!("Failed to execute query: {:?}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!("New subscriber details have been saved");
    StatusCode::OK
}
