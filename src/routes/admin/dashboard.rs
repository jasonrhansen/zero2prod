use askama::Template;
use axum::{extract::State, response::Html, Extension};

use crate::{
    app_error::AppError, app_state::AppState, authentication::UserId, email_client::EmailClient,
};

use super::get_username;

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
struct AdminDashboardTemplate {
    username: String,
}

pub async fn admin_dashboard<E>(
    State(state): State<AppState<E>>,
    Extension(user_id): Extension<UserId>,
) -> Result<Html<String>, AppError>
where
    E: EmailClient + Clone + 'static,
{
    let username = get_username(*user_id, &state.db_pool).await?;
    let template = AdminDashboardTemplate { username };
    Ok(Html(template.render().unwrap()))
}
