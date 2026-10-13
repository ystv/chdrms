use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{
    asset::permission,
    bundle::{self as database, CreateBundle},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::{asset::model::AssetDto, bundle::model::AssetBundleDto},
    state::AppState,
};

pub(super) mod model;

pub(super) const TAG: &str = "asset_bundle";

/// Get an asset bundle by its ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_asset_bundle_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset bundle ID"),
    ),
    responses(
        (status = OK, description = "Success", body = AssetBundleDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset bundle by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetBundleDto>> {
    Ok(Json(
        database::Bundle::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)
            .map(|bundle| AssetBundleDto { id: bundle.id })?,
    ))
}

/// Get assets within an asset bundle.
#[utoipa::path(
    get,
    path = "/{id}/assets",
    tag = TAG,
    operation_id = "get_assets_within_bundle_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset bundle ID"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<AssetDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset bundle by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_assets_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AssetDto>>> {
    let mut txn = state.transaction().await?;

    let assets = database::Bundle::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .assets(&mut txn)
        .await?;

    Ok(Json(
        super::asset::populate_asset_dtos(assets, &mut txn).await?,
    ))
}

/// Create an asset bundle.
#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    operation_id = "create_asset_bundle",
    responses(
        (status = OK, description = "Success", body = Vec<AssetDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<permission::Manage>,
    auth: AuthContext,
) -> Result<Json<AssetBundleDto>> {
    let mut txn = state.transaction().await?;

    let bundle = database::Bundle::create(
        &mut txn,
        CreateBundle {
            created_by: auth.user().id,
        },
    )
    .await
    .map(|bundle| AssetBundleDto { id: bundle.id })?;

    txn.commit().await?;

    Ok(Json(bundle))
}

/// Delete an asset bundle.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    operation_id = "delete_asset_bundle",
    params(
        ("id" = Uuid, Path, description = "Requested asset bundle ID"),
    ),
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset bundle by that ID not found", body = ErrorResponse)
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;

    database::Bundle::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

/// List all asset bundles.
#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "list_asset_bundles",
    responses(
        (status = OK, description = "Success", body = Vec<AssetBundleDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset bundle by that ID not found", body = ErrorResponse)
    ),
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<permission::View>,
) -> Result<Json<Vec<AssetBundleDto>>> {
    Ok(Json(
        database::Bundle::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(|bundle| AssetBundleDto { id: bundle.id })
            .collect(),
    ))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id))
        .routes(routes!(get_assets_by_id))
        .routes(routes!(create))
        .routes(routes!(delete))
        .routes(routes!(list))
}
