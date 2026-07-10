use std::collections::{HashMap, HashSet};

use axum::{
    Json,
    extract::{Path, State},
};
use chdrms_database::user::{self, User};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    state::AppState,
};

pub(super) const TAG: &str = "user";

#[derive(Serialize, ToSchema)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
}

impl From<&User> for UserDto {
    fn from(value: &User) -> Self {
        Self {
            id: value.id,
            email: value.email.clone(),
            name: value.name.clone(),
            is_admin: value.is_admin,
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
    operation_id = "get_current_user",
    responses(
        (status = OK, description = "Success", body = UserDto),
    ),
)]
async fn current_user(auth: AuthContext) -> Json<UserDto> {
    Json(auth.user().into())
}

/// Get a user by their ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_user_by_id",
    responses(
        (status = OK, description = "Success", body = UserDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "User by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<user::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserDto>> {
    Ok(Json(
        (&user::User::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?)
            .into(),
    ))
}

/// Get the current user's permissions.
#[utoipa::path(
    get,
    path = "/@me/permissions",
    tag = TAG,
    operation_id = "get_current_user_permissions",
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
    operation_id = "list_users",
    responses(
        (status = OK, description = "Success", body = [UserDto])
    )
)]
async fn list_users(
    State(state): State<AppState>,
    _auth: RequirePermission<user::permission::List>,
) -> Result<Json<Vec<UserDto>>> {
    Ok(Json(
        User::list(&mut state.transaction().await?)
            .await?
            .iter()
            .map(From::from)
            .collect(),
    ))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(current_user))
        .routes(routes!(current_user_permissions))
        .routes(routes!(list_users))
        .routes(routes!(get_by_id))
}
