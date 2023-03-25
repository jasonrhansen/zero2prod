use anyhow::Context;
use async_session::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts, Extension};
use axum_sessions::SessionHandle;
use tokio::sync::OwnedRwLockWriteGuard;
use uuid::Uuid;

#[derive(Debug)]
pub struct TypedSession {
    session: OwnedRwLockWriteGuard<async_session::Session>,
}

impl TypedSession {
    const USER_ID_KEY: &str = "user_id";

    pub fn regenerate(&mut self) {
        self.session.regenerate();
    }

    pub fn insert_user_id(&mut self, user_id: Uuid) -> Result<(), anyhow::Error> {
        self.session
            .insert(Self::USER_ID_KEY, user_id)
            .context("Unable to save user_id to session")
    }

    pub fn get_user_id(&self) -> Option<Uuid> {
        self.session.get::<Uuid>(Self::USER_ID_KEY)
    }

    pub fn log_out(&mut self) {
        self.session.destroy();
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for TypedSession
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request_parts(parts, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.write_owned().await;

        Ok(Self { session })
    }
}
