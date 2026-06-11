use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use chdrms_database::user::User;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub mod oidc;

macro_rules! cookie_name {
    ($name:expr) => {
        concat!("rms_", $name)
    };
}

use cookie_name;

const SESSION_COOKIE: &str = cookie_name!("session");

// We are generic here, rather than being specifically for users, so we can support API tokens later
pub enum AuthContext {
    User { user: User },
}

impl FromRequestParts<AppState> for AuthContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut txn = state.transaction().await?;

        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(cookie) = jar.get(SESSION_COOKIE) else {
            return Err(AppError::Unauthorized);
        };

        let Ok(token) = Uuid::parse_str(cookie.value()) else {
            return Err(AppError::Unauthorized);
        };

        let Some(user) = User::get_by_session(&mut txn, token).await? else {
            return Err(AppError::Unauthorized);
        };

        Ok(Self::User { user })
    }
}
