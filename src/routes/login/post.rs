use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_flash::Flash;
use hyper::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    authentication::{validate_credentials, AuthError, Credentials},
    email_client::EmailClient,
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

#[tracing::instrument(
    skip(form, state),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login<E>(
    flash: Flash,
    State(state): State<AppState<E>>,
    Form(form): Form<FormData>,
) -> (Flash, Redirect)
where
    E: EmailClient + Clone + 'static,
{
    let credentials: Credentials = form.into();

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    match validate_credentials(credentials, &state.db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            (flash, Redirect::to("/"))
        }
        Err(e) => {
            let error = match e {
                AuthError::InvalidCredentials(_) => LoginError::Auth(e.into()),
                AuthError::Unexpected(_) => LoginError::Unexpected(e.into()),
            };

            (flash.error(error.to_string()), Redirect::to("/login"))
        }
    }
}
