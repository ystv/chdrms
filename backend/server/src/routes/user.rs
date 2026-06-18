use std::collections::{HashMap, HashSet};

use axum::{Json, extract::State};
use chdrms_database::user::{self, User};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::Result,
    state::AppState,
};

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

#[derive(Serialize, ToSchema)]
pub struct ObjectPermissions {
    name: String,
    actions: Vec<String>,
}

impl ObjectPermissions {
    pub fn from_map(map: &HashMap<String, HashSet<String>>) -> Vec<Self> {
        let mut permissions: Vec<_> = map
            .iter()
            .map(|(object, permissions)| {
                let mut actions: Vec<_> = permissions.iter().cloned().collect();
                actions.sort();
                ObjectPermissions {
                    name: object.clone(),
                    actions,
                }
            })
            .collect();
        permissions.sort_by(|o1, o2| o1.name.cmp(&o2.name));
        permissions
    }
}

/// Get current user.
#[utoipa::path(
    get,
    path = "/@me",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = UserInfo),
    ),
)]
async fn current_user(auth: AuthContext) -> Json<UserInfo> {
    Json(auth.user().into())
}

/// Get the current user's permissions.
#[utoipa::path(
    get,
    path = "/@me/permissions",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [ObjectPermissions]),
    ),
)]
async fn current_user_permissions(auth: AuthContext) -> Json<Vec<ObjectPermissions>> {
    Json(ObjectPermissions::from_map(auth.permissions()))
}

/// List all users.
#[utoipa::path(
    get,
    path = "",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [UserInfo])
    )
)]
async fn list_users(
    State(state): State<AppState>,
    _auth: RequirePermission<user::permission::List>,
) -> Result<Json<Vec<UserInfo>>> {
    Ok(Json(
        User::list(&mut state.transaction().await?)
            .await?
            .iter()
            .map(From::from)
            .collect(),
    ))
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(current_user))
        .routes(routes!(current_user_permissions))
        .routes(routes!(list_users))
}
