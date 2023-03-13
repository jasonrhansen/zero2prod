use axum::extract::FromRef;
use sqlx::PgPool;

use crate::email_client::EmailClient;

#[derive(Clone)]
pub struct AppState<E>
where
    E: EmailClient + Clone,
{
    pub db_pool: PgPool,
    pub email_client: E,
    pub base_url: String,
    pub flash_config: axum_flash::Config,
}

impl<E> FromRef<AppState<E>> for axum_flash::Config
where
    E: EmailClient + Clone + 'static,
{
    fn from_ref(state: &AppState<E>) -> Self {
        state.flash_config.clone()
    }
}
