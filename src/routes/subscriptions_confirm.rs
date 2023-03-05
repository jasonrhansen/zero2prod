use axum::extract::{Query, State};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{app_error::AppError, app_state::AppState, email_client::EmailClient};

#[derive(Deserialize)]
pub struct SubscriptionsConfirmParams {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(state, params))]
pub async fn confirm<E>(
    State(state): State<AppState<E>>,
    Query(params): Query<SubscriptionsConfirmParams>,
) -> Result<StatusCode, AppError>
where
    E: EmailClient + Clone,
{
    let subscriber_id =
        get_subscriber_id_from_token(&state.db_pool, &params.subscription_token).await?;

    let subscriber_id = match subscriber_id {
        Some(id) => id,
        None => return Ok(StatusCode::UNAUTHORIZED),
    };

    confirm_subscriber(&state.db_pool, subscriber_id).await?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions set status = 'confirmed'
        WHERE id = $1
        "#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update subscription status: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}
