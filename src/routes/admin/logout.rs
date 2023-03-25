use axum::response::Redirect;
use axum_flash::Flash;

use crate::session_state::TypedSession;

pub async fn log_out(mut session: TypedSession, flash: Flash) -> (Flash, Redirect) {
    let flash = if session.get_user_id().is_some() {
        session.log_out();
        flash.info("You have successfully logged out.")
    } else {
        flash
    };

    (flash, Redirect::to("/login"))
}
