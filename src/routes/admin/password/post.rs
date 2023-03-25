use axum::{extract::State, response::Redirect, Form};
use axum_flash::Flash;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::{
    app_error::AppError,
    app_state::AppState,
    authentication::{self, validate_credentials, AuthError, Credentials},
    email_client::EmailClient,
    routes::get_username,
    session_state::TypedSession,
};

#[derive(Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password<E>(
    flash: Flash,
    State(state): State<AppState<E>>,
    session: TypedSession,
    Form(form): Form<FormData>,
) -> Result<(Flash, Redirect), AppError>
where
    E: EmailClient + Clone + 'static,
{
    let user_id = match session.get_user_id() {
        Some(user_id) => user_id,
        None => return Ok((flash, Redirect::to("/login"))),
    };

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        let flash_message =
            "You entered two different new passwords - the field values must match.";
        return Ok((flash.error(flash_message), Redirect::to("/admin/password")));
    }

    let username = get_username(user_id, &state.db_pool).await?;
    let credentials = Credentials {
        username,
        password: form.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &state.db_pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                let flash_message = "The current password is incorrect.";
                return Ok((flash.error(flash_message), Redirect::to("/admin/password")));
            }
            AuthError::Unexpected(_) => Err(e.into()),
        };
    }

    authentication::change_password(user_id, form.new_password, &state.db_pool).await?;

    Ok((
        flash.info("Your password has been changed."),
        Redirect::to("/admin/password"),
    ))
}
