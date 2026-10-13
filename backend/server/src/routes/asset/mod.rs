//! This routing module contains the following dedicated field endpoints:
//! * `GET /asset/{id}/location`
//! * `POST /asset/{id}/location`
//!
//! The location field of assets is given a dedicated endpoint as we may
//! apply side-effects like location tracking/auditing in the future,
//! which - by specification - breaks [idempotence](https://datatracker.ietf.org/doc/html/rfc7231#section-4.2.2).
//!
//! None of the other fields plan to have this level of tracking, so
//! should be updated via the regular PUT/PATCH methods.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{
    asset::{self as database, CreateAsset, UpdateAsset},
    comment::{CreateComment, UpdateComment},
};

use sqlx::types::chrono::Utc;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::{
        asset::model::{
            AssetDto, AssetLocations, CreateAssetRequest, TimelineEventDto, TimelineEventTypeDto,
            UpdateAssetLocationRequest, UpdateAssetRequest,
        },
        location::LocationDto,
        model::{AssetIdentifier, CommentDto, CreateCommentRequest, UpdateCommentRequest},
    },
    state::AppState,
};

pub(super) mod model;

pub(super) const TAG: &str = "asset";

/// Get an asset by its ID or Tag.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_asset_by_id_or_tag",
    params(
        ("id", Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = AssetDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID or Tag not found", body = ErrorResponse)
    ),
)]
async fn get_by_id_or_tag(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<AssetIdentifier>,
) -> Result<Json<AssetDto>> {
    let mut txn = state.transaction().await?;
    let asset = match id {
        AssetIdentifier::Id(id) => database::Asset::get_by_id(&mut txn, id).await,
        AssetIdentifier::Tag(tag) => database::Asset::get_by_tag(&mut txn, tag).await,
    }?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(AssetDto {
        id: asset.id,
        r#type: asset.r#type,
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle,
        locations: AssetLocations {
            current: asset.location,
            home: asset.home_location,
        },
    }))
}

/// Create a new asset.
#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    operation_id = "create_asset",
    responses(
        (status = OK, description = "Success", body = AssetDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    auth: AuthContext,
    Json(asset): Json<CreateAssetRequest>,
) -> Result<Json<AssetDto>> {
    let mut txn = state.transaction().await?;

    let asset = database::Asset::create(
        &mut txn,
        CreateAsset {
            r#type: asset.r#type,
            alias: asset.alias,
            notes: asset.notes,
            tag: asset.tag,
            bundle: asset.bundle,
            home_location: asset.locations.home,
            location: asset.locations.current,
            created_by: auth.user().id,
        },
    )
    .await?;

    txn.commit().await?;

    Ok(Json(AssetDto {
        id: asset.id,
        r#type: asset.r#type,
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle,
        locations: AssetLocations {
            current: asset.location,
            home: asset.home_location,
        },
    }))
}

/// Update an asset by its ID.
#[utoipa::path(
    put,
    path = "/{id}",
    tag = TAG,
    operation_id = "update_asset_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn update(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateAssetRequest>,
) -> Result<Json<AssetDto>> {
    let mut txn = state.transaction().await?;

    let asset = database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    let location = asset.location;
    let asset = asset
        .update(
            &mut txn,
            UpdateAsset {
                alias: update.alias,
                notes: update.notes,
                tag: update.tag,
                bundle: update.bundle,
                home_location: update.locations.home,
                location,
            },
        )
        .await?;

    Ok(Json(AssetDto {
        id: asset.id,
        r#type: asset.r#type,
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle,
        locations: AssetLocations {
            current: asset.location,
            home: asset.home_location,
        },
    }))
}

/// List all assets.
#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "list_assets",
    responses(
        (status = OK, description = "Success", body = Vec<AssetDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
) -> Result<Json<Vec<AssetDto>>> {
    Ok(Json(
        database::Asset::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(|asset| AssetDto {
                id: asset.id,
                r#type: asset.r#type,
                alias: asset.alias,
                notes: asset.notes,
                tag: asset.tag,
                bundle: asset.bundle,
                locations: AssetLocations {
                    current: asset.location,
                    home: asset.home_location,
                },
            })
            .collect(),
    ))
}

/// Delete an asset by its ID.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    operation_id = "delete_asset_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    // todo: this route does not do a recursive deletion of
    //       relationships to the asset. this is mainly to
    //       align with DELETE's idempotence. however, it is
    //       currently not possible to delete comments, in
    //       attempts to preserve history. how should this be
    //       handled?

    let mut txn = state.transaction().await?;

    database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get an asset's location
#[utoipa::path(
    get,
    path = "/{id}/location",
    tag = TAG,
    operation_id = "get_asset_location",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = LocationDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_location(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<Json<LocationDto>> {
    let mut txn = state.transaction().await?;

    Ok(Json(
        database::Asset::get_by_id(&mut txn, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .get_location(&mut txn)
            .await
            .map(Into::into)?,
    ))
}

/// Set an asset's location.
#[utoipa::path(
    post,
    path = "/{id}/location",
    tag = TAG,
    operation_id = "set_asset_location",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = AssetDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn put_location(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(UpdateAssetLocationRequest { location }): Json<UpdateAssetLocationRequest>,
) -> Result<Json<AssetDto>> {
    let mut txn = state.transaction().await?;

    let asset = database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .set_location(&mut txn, location)
        .await?;

    txn.commit().await?;

    Ok(Json(AssetDto {
        id: asset.id,
        r#type: asset.r#type,
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle,
        locations: AssetLocations {
            current: asset.location,
            home: asset.home_location,
        },
    }))
}

/// List comments attached to this asset.
#[utoipa::path(
    get,
    path = "/{id}/comments",
    tag = TAG,
    operation_id = "list_asset_comments_by_asset_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<CommentDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn list_comments(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>, // todo: special permission for comments?
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CommentDto>>> {
    let mut txn = state.transaction().await?;

    Ok(Json(
        database::Asset::get_by_id(&mut txn, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .list_comments(&mut txn)
            .await?
            .into_iter()
            .map(|comment| CommentDto {
                id: comment.id,
                archived_at: comment.archived_at,
                title: comment.title,
                content: comment.content,
                created_at: comment.created_at,
                created_by: comment.created_by,
            })
            .collect(),
    ))
}

/// Add a comment to the specified asset.
#[utoipa::path(
    post,
    path = "/{id}/comments",
    tag = TAG,
    operation_id = "create_asset_comment_by_asset_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<CommentDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn create_comment(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>, // todo: special permission for comments?
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(comment): Json<CreateCommentRequest>,
) -> Result<Json<CommentDto>> {
    let mut txn = state.transaction().await?;

    let comment = database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .add_comment(
            &mut txn,
            CreateComment {
                title: comment.title,
                content: comment.content,
                created_by: auth.user().id,
            },
        )
        .await?;

    txn.commit().await?;

    Ok(Json(CommentDto {
        id: comment.id,
        archived_at: comment.archived_at,
        title: comment.title,
        content: comment.content,
        created_at: comment.created_at,
        created_by: comment.created_by,
    }))
}

/// Update a comment attached to an asset based on its ID.
#[utoipa::path(
    put,
    path = "/{asset_id}/comments/{comment_id}",
    tag = TAG,
    operation_id = "update_asset_comment_by_asset_and_comment_id",
    params(
        ("asset_id" = Uuid, Path, description = "Requested asset ID"),
        ("comment_id" = Uuid, Path, description = "Requested comment ID"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<CommentDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset or comment by that ID not found", body = ErrorResponse)
    ),
)]
async fn update_comment(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>, // todo: special permission for comments?
    Path((asset, comment)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateCommentRequest>,
) -> Result<Json<CommentDto>> {
    let mut txn = state.transaction().await?;

    let comment = database::Asset::get_by_id(&mut txn, asset)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .get_comment_by_id(&mut txn, comment)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    let archived_at = match (comment.archived_at.is_some(), request.archived) {
        (true, true) => comment.archived_at,
        (false, true) => Some(Utc::now()),
        (_, false) => None,
    };

    let comment = comment
        .update(
            &mut txn,
            UpdateComment {
                archived_at,
                title: request.title,
                content: request.content,
            },
        )
        .await?;

    txn.commit().await?;

    Ok(Json(CommentDto {
        id: comment.id,
        archived_at: comment.archived_at,
        title: comment.title,
        content: comment.content,
        created_at: comment.created_at,
        created_by: comment.created_by,
    }))
}

#[utoipa::path(
    get,
    path = "/{id}/timeline",
    tag = TAG,
    operation_id = "get_asset_timeline_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<TimelineEventDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_timeline(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TimelineEventDto>>> {
    // todo: this logic should probably be moved out of the router,
    //       given it will be slightly more advance than a single
    //       database lookup in the future.

    let mut txn = state.transaction().await?;
    let mut events = Vec::new();

    let asset = database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    // fetch comments
    events.extend(
        asset
            .list_comments(&mut txn)
            .await?
            .into_iter()
            .map(|comment| TimelineEventDto {
                title: comment.title,
                content: comment.content,
                time: comment.created_at,
                r#type: TimelineEventTypeDto::Comment {
                    comment: comment.id,
                },
            }),
    );

    // sort by date
    events.sort_by_key(|event| event.time);

    Ok(Json(events))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id_or_tag))
        .routes(routes!(create))
        .routes(routes!(list))
        .routes(routes!(delete))
        .routes(routes!(update))
        .routes(routes!(get_location))
        .routes(routes!(put_location))
        .routes(routes!(list_comments))
        .routes(routes!(update_comment))
        .routes(routes!(create_comment))
        .routes(routes!(get_timeline))
}
