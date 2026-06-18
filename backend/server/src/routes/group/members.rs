use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{
    user::User,
    user_group::{self, Group},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::permissions::RequirePermission,
    error::{AppError, Result},
    routes::user::UserInfo,
    state::AppState,
};

#[derive(Serialize, ToSchema)]
pub struct GroupMembers {
    members: Vec<UserInfo>,
}

#[derive(Deserialize, ToSchema)]
pub struct ModifyMember {
    member: Uuid,
}

/// List group members.
#[utoipa::path(
    get,
    path = "/{group_id}/members",
    tag = super::TAG,
    responses(
        (status = OK, description = "Success", body = GroupMembers),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManageMembers>,
) -> Result<Json<GroupMembers>> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };
    let members = group
        .list_members(&mut txn)
        .await?
        .iter()
        .map(From::from)
        .collect();
    Ok(Json(GroupMembers { members }))
}

/// Add user to group.
#[utoipa::path(
    post,
    path = "/{group_id}/members",
    tag = super::TAG,
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn add(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManageMembers>,
    Json(member): Json<ModifyMember>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };

    let Some(user) = User::get_by_id(&mut txn, member.member).await? else {
        return Err(AppError::bad_request("user not found"));
    };

    group.add_member(&mut txn, user).await?;

    txn.commit().await?;

    Ok(StatusCode::CREATED) // TODO: return a body?
}

/// Remove user from group.
#[utoipa::path(
    delete,
    path = "/{group_id}/members",
    tag = super::TAG,
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::ManageMembers>,
    Json(member): Json<ModifyMember>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    let Some(group) = Group::get_by_id(&mut txn, id).await? else {
        return Err(AppError::NotFound);
    };

    let Some(user) = User::get_by_id(&mut txn, member.member).await? else {
        return Err(AppError::bad_request("user not found"));
    };

    let removed = group.remove_member(&mut txn, user).await?;

    txn.commit().await?;

    Ok(if removed {
        StatusCode::CREATED
    } else {
        StatusCode::BAD_REQUEST
    }) // TODO: return a body?
}
