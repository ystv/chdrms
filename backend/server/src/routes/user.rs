use axum::Json;
use chdrms_database::user::User;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{auth::AuthContext, state::AppState};

pub(super) const TAG: &str = "user";

#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl From<&User> for UserInfo {
    fn from(value: &User) -> Self {
        Self {
            id: value.id,
            email: value.email.clone(),
            name: value.name.clone(),
        }
    }
}

/// Get current user.
#[utoipa::path(
    get,
    path = "/@me",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [UserInfo]),
    ),
)]
async fn current_user(auth: AuthContext) -> Json<UserInfo> {
    Json(auth.user().into())
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(current_user))
}
