use std::collections::{HashMap, HashSet};

use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use chdrms_database::user::User;
use uuid::Uuid;

use crate::{auth::permissions::build_permission_map, error::AppError, state::AppState};

pub mod oidc;
pub mod permissions;

macro_rules! cookie_name {
    ($name:expr) => {
        concat!("rms_", $name)
    };
}

use cookie_name;

pub const SESSION_COOKIE: &str = cookie_name!("session");

// We are generic here, rather than being specifically for users, so we can support API tokens later
pub enum AuthContext {
    User {
        user: User,
        permissions: HashMap<String, HashSet<String>>,
    },
}

impl AuthContext {
    pub fn user(&self) -> &User {
        match self {
            AuthContext::User { user, .. } => user,
        }
    }

    pub fn permissions(&self) -> &HashMap<String, HashSet<String>> {
        match self {
            AuthContext::User { permissions, .. } => permissions,
        }
    }

    pub fn has_permission_raw(&self, object: &str, action: &str) -> bool {
        if self.user().is_admin {
            return true;
        }
        let permissions = self.permissions();
        let Some(actions) = permissions.get(object) else {
            return false;
        };
        actions.contains(action)
    }
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

        let permissions = user.list_permissions(&mut txn).await?;
        let permissions = build_permission_map(permissions);

        Ok(Self::User { user, permissions })
    }
}
