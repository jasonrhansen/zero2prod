use std::ops::Deref;

use axum::{
    middleware::Next,
    response::{Redirect, Response},
};
use hyper::Request;
use uuid::Uuid;

use crate::session_state::TypedSession;

pub async fn reject_anonymous_users<B>(
    session: TypedSession,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Redirect> {
    if let Some(user_id) = session.get_user_id() {
        request.extensions_mut().insert(UserId(user_id));
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(Redirect::to("/login"))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
