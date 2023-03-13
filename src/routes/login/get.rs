use axum::response::Html;
use axum_flash::IncomingFlashes;

pub async fn login_form(flashes: IncomingFlashes) -> (IncomingFlashes, Html<String>) {
    let body_text = if !flashes.is_empty() {
        let mut body_text = String::new();
        for (_level, text) in &flashes {
            body_text += &format!("<p><i>{}</i></p>", text);
        }
        body_text
    } else {
        include_str!("login.html").to_owned()
    };

    (flashes, Html(body_text))
}
