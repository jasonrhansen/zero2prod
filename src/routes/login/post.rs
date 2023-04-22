use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_flash::Flash;
use hyper::StatusCode;
use secrecy::Secret;
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    app_state::AppState,
    authentication::{validate_credentials, AuthError, Credentials},
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

impl From<FormData> for Credentials {
    fn from(FormData { username, password }: FormData) -> Self {
        Self { username, password }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Authentication failed")]
    Auth(#[source] anyhow::Error),
    #[error("Something went wrong")]
    Unexpected(#[from] anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            LoginError::Auth(_) => StatusCode::UNAUTHORIZED,
            LoginError::Unexpected(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}

impl From<AuthError> for LoginError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials(e) => LoginError::Auth(e),
            AuthError::Unexpected(e) => LoginError::Unexpected(e),
        }
    }
}

#[tracing::instrument(
    skip(form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    flash: Flash,
    State(state): State<AppState>,
    session: TypedSession,
    Form(form): Form<FormData>,
) -> (Flash, Redirect) {
    let credentials: Credentials = form.into();
    match try_login(credentials, &state.db_pool, session).await {
        Ok(()) => (flash, Redirect::to("/admin/dashboard")),
        Err(e) => (flash.error(e.to_string()), Redirect::to("/login")),
    }
}

async fn try_login(
    credentials: Credentials,
    db_pool: &PgPool,
    mut session: TypedSession,
) -> Result<(), LoginError> {
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, db_pool).await?;
    session.regenerate();
    Ok(session.insert_user_id(user_id)?)
}
