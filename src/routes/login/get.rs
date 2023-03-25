use askama::Template;
use axum::response::Html;
use axum_flash::IncomingFlashes;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    flashes: IncomingFlashes,
}

pub async fn login_form(flashes: IncomingFlashes) -> (IncomingFlashes, Html<String>) {
    let template = LoginTemplate {
        flashes: flashes.clone(),
    };

    (flashes, Html(template.render().unwrap()))
}
