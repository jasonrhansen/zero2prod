use sqlx::PgPool;

use crate::email_client::EmailClient;

#[derive(Clone)]
pub struct AppState<E>
where
    E: EmailClient + Clone,
{
    pub connection_pool: PgPool,
    pub email_client: E,
}
