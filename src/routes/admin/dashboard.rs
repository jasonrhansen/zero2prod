use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::{
    app_error::AppError, app_state::AppState, email_client::EmailClient,
    session_state::TypedSession,
};

use super::get_username;

pub async fn admin_dashboard<E>(
    State(state): State<AppState<E>>,
    session: TypedSession,
) -> Result<Response, AppError>
where
    E: EmailClient + Clone + 'static,
{
    let response = if let Some(user_id) = session.get_user_id() {
        let username = get_username(user_id, &state.db_pool).await?;
        Html(format!(
            r#"<!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta http-equiv="content-type" content="text/html; charset=utf-8">
                    <title>Admin dashboard</title>
                </head>
                <body>
                    <p>Welcome {username}!</p>
                    <p>Available actions:</p>
                    <ol>
                        <li><a href="/admin/password">Change password</a></li>
                        <li>
                            <form name="logoutForm" action="/admin/logout" method="post">
                                <input type="submit" value="Logout">
                            </form>
                        </li>
                    </ol>
                </body>
            </html>"#
        ))
        .into_response()
    } else {
        Redirect::to("/login").into_response()
    };

    Ok(response)
}
