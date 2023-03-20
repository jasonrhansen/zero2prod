use anyhow::Context;
use axum::extract::State;
use axum_sessions::extractors::ReadableSession;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{app_error::AppError, app_state::AppState, email_client::EmailClient};

pub async fn admin_dashboard<E>(
    State(state): State<AppState<E>>,
    session: ReadableSession,
) -> Result<String, AppError>
where
    E: EmailClient + Clone + 'static,
{
    let user_id = session.get::<Uuid>("user_id").context("Not logged in")?;
    let username = get_username(user_id, &state.db_pool).await?;

    Ok(format!(
        r#"<!DOCTYPE html>
        <html lang="en">
            <head>
                <meta http-equiv="content-type" content="text/html; charset=utf-8">
                <title>Admin dashboard</title>
            </head>
            <body>
                <p>Welcome {username}!</p>
            </body>
        </html>"#
    ))
}

async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to retreive username from database")?;

    Ok(row.username)
}
