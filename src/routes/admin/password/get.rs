use askama::Template;
use axum::response::Html;
use axum_flash::IncomingFlashes;

#[derive(Template)]
#[template(path = "change_password.html")]
struct ChangePasswordTemplate {
    flashes: IncomingFlashes,
}

pub async fn change_password_form(flashes: IncomingFlashes) -> (IncomingFlashes, Html<String>) {
    let template = ChangePasswordTemplate {
        flashes: flashes.clone(),
    };

    (flashes, Html(template.render().unwrap()))
}
