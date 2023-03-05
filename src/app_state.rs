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
}
