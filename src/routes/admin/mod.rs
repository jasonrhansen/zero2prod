mod dashboard;
mod logout;
mod password;

pub use dashboard::admin_dashboard;
pub use logout::log_out;
pub use password::*;

use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
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
