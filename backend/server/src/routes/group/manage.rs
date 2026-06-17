use axum::{
    Json,
    extract::{Path, State},
};
use chdrms_database::user_group::{self, Group};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::permissions::RequirePermission,
    error::{AppError, Result},
    state::AppState,
};

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

/// List groups.
#[utoipa::path(
    get,
    path = "",
    tag = super::TAG,
    responses(
        (status = OK, description = "Success", body = [Vec<GroupInfo>]),
    ),
)]
pub(super) async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<user_group::permission::List>,
) -> Result<Json<Vec<GroupInfo>>> {
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
    tag = super::TAG,
    responses(
        (status = OK, description = "Success", body = [GroupInfo]),
    ),
)]
pub(super) async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<user_group::permission::Manage>,
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
    tag = super::TAG,
    responses(
        (status = OK, description = "Success", body = [GroupInfo]),
        (status = NOT_FOUND, description = "Not found"),
    ),
)]
pub(super) async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<user_group::permission::List>,
) -> Result<Json<GroupInfo>> {
    let mut txn = state.transaction().await?;
    Group::get_by_id(&mut txn, id)
        .await?
        .map(|g| Json((&g).into()))
        .ok_or_else(|| AppError::NotFound)
}
