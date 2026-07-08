use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_valid::Validated;
use chdrms_database::user_group::{self, Group};
use uuid::Uuid;

use crate::{
    auth::permissions::{PermissionRef, RequirePermission, build_permission_map},
    error::{AppError, Result},
    routes::user::ObjectPermissions,
    state::AppState,
};

/// List group permissions.
#[utoipa::path(
    get,
    path = "/{group_id}/permissions",
    tag = super::TAG,
    operation_id = "list_group_permissions",
    responses(
        (status = OK, description = "Success", body = [ObjectPermissions]),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManagePermissions>,
) -> Result<Json<Vec<ObjectPermissions>>> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };
    let permissions = group.list_permissions(&mut txn).await?;
    let permissions = build_permission_map(permissions);
    Ok(Json(ObjectPermissions::from_map(&permissions)))
}

/// Grant permission to group.
#[utoipa::path(
    post,
    path = "/{group_id}/permissions",
    tag = super::TAG,
    operation_id = "add_permission_to_group",
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn add(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManageMembers>,
    Validated(Json(permission)): Validated<Json<PermissionRef>>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };

    group
        .add_permission(&mut txn, &permission.object, &permission.action)
        .await?;

    txn.commit().await?;

    Ok(StatusCode::CREATED) // TODO: return a body?
}

/// Revoke permission from group.
#[utoipa::path(
    delete,
    path = "/{group_id}/permissions",
    tag = super::TAG,
    operation_id = "remove_permission_from_group",
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManageMembers>,
    Validated(Json(permission)): Validated<Json<PermissionRef>>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };

    let removed = group
        .remove_permission(&mut txn, &permission.object, &permission.action)
        .await?;

    txn.commit().await?;

    Ok(if removed {
        StatusCode::CREATED
    } else {
        StatusCode::BAD_REQUEST
    }) // TODO: return a body?
}
