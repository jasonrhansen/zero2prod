use axum::response::Redirect;
use axum_flash::Flash;

pub async fn log_out(flash: Flash) -> (Flash, Redirect) {
    (
        flash.info("You have successfully logged out."),
        Redirect::to("/login"),
    )
}
