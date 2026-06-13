use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{user::User, user_group::Group};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::AuthContext,
    error::{AppError, Result},
    routes::user::UserInfo,
    state::AppState,
};

pub(super) const TAG: &str = "group";

#[derive(Serialize, ToSchema)]
pub struct GroupInfo {
    pub id: Uuid,
    pub name: String,
}

impl From<&Group> for GroupInfo {
    fn from(value: &Group) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateGroup {
    name: String,
}

#[derive(Serialize, ToSchema)]
pub struct GroupMembers {
    members: Vec<UserInfo>,
}

#[derive(Deserialize, ToSchema)]
pub struct ModifyMember {
    member: Uuid,
}

/// List groups.
#[utoipa::path(
    get,
    path = "",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [Vec<GroupInfo>]),
    ),
)]
async fn list(State(state): State<AppState>, _auth: AuthContext) -> Result<Json<Vec<GroupInfo>>> {
    let mut txn = state.transaction().await?;
    Ok(Json(
        Group::list(&mut txn)
            .await?
            .iter()
            .map(From::from)
            .collect(),
    ))
}

/// Create group.
#[utoipa::path(
    post,
    path = "",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [GroupInfo]),
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: AuthContext,
    Json(create): Json<CreateGroup>,
) -> Result<Json<GroupInfo>> {
    let mut txn = state.transaction().await?;
    let group = Group::create(&mut txn, &create.name).await?;
    txn.commit().await?;
    Ok(Json((&group).into()))
}

/// Get group.
#[utoipa::path(
    get,
    path = "/{group_id}",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [GroupInfo]),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<GroupInfo>> {
    let mut txn = state.transaction().await?;
    Group::get_by_id(&mut txn, id)
        .await?
        .map(|g| Json((&g).into()))
        .ok_or_else(|| AppError::NotFound)
}

/// List group members.
#[utoipa::path(
    get,
    path = "/{group_id}/members",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [GroupMembers]),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
async fn list_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthContext,
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
    tag = TAG,
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
async fn add_member(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthContext,
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
    tag = TAG,
    responses(
        (status = CREATED, description = "Success"),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
async fn remove_member(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: AuthContext,
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

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(get))
        .routes(routes!(list_members, add_member, remove_member))
}
