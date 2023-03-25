use askama::Template;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_flash::IncomingFlashes;

use crate::session_state::TypedSession;

#[derive(Template)]
#[template(path = "change_password.html")]
struct ChangePasswordTemplate {
    flashes: Vec<String>,
}

pub async fn change_password_form(flashes: IncomingFlashes, session: TypedSession) -> Response {
    if session.get_user_id().is_none() {
        return Redirect::to("/login").into_response();
    }
    let template = ChangePasswordTemplate {
        flashes: flashes
            .into_iter()
            .map(|(_level, msg)| msg.to_string())
            .collect(),
    };

    Html(template.render().unwrap()).into_response()
}
