use axum::{extract::State, response::Html, Extension};

use crate::{
    app_error::AppError, app_state::AppState, authentication::UserId, email_client::EmailClient,
};

use super::get_username;

pub async fn admin_dashboard<E>(
    State(state): State<AppState<E>>,
    Extension(user_id): Extension<UserId>,
) -> Result<Html<String>, AppError>
where
    E: EmailClient + Clone + 'static,
{
    let username = get_username(*user_id, &state.db_pool).await?;
    Ok(Html(format!(
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
    )))
}
