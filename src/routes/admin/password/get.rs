use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use axum_flash::IncomingFlashes;

#[derive(Template)]
#[template(path = "change_password.html")]
struct ChangePasswordTemplate {
    flashes: Vec<String>,
}

pub async fn change_password_form(flashes: IncomingFlashes) -> Response {
    let template = ChangePasswordTemplate {
        flashes: flashes
            .into_iter()
            .map(|(_level, msg)| msg.to_string())
            .collect(),
    };

    Html(template.render().unwrap()).into_response()
}
