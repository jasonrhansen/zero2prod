use axum::{
    middleware::Next,
    response::{Redirect, Response},
};
use hyper::Request;

use crate::session_state::TypedSession;

pub async fn reject_anonymous_users<B>(
    session: TypedSession,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Redirect> {
    if let Some(user_id) = session.get_user_id() {
        request.extensions_mut().insert(user_id);
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(Redirect::to("/login"))
    }
}
