use axum::extract::FromRef;
use sqlx::PgPool;

use crate::email_client::DynEmailClient;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db_pool: PgPool,
    pub email_client: DynEmailClient,
    pub base_url: String,
    pub flash_config: axum_flash::Config,
}
