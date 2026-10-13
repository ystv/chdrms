use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{
    asset::{AssetRepository, permission},
    bundle::{self as database, BundleRepository},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::{
        asset::model::{AssetDto, AssetLocations},
        bundle::model::AssetBundleDto,
    },
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
    use database::GetBundleByIdError;

    state
        .repository
        .get_bundle_by_id(id)
        .await
        .map_err(|err| match err {
            GetBundleByIdError::Backend => AppError::internal_server_error("internal server error"),
        })?
        .map(|bundle| Json(AssetBundleDto { id: bundle.id }))
        .ok_or_else(|| AppError::NotFound)
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
    use chdrms_database::asset::GetAssetsByBundleIdError;
    use database::GetBundleByIdError;

    // check if bundle exists
    let id = state
        .repository
        .get_bundle_by_id(id)
        .await
        .map_err(|err| match err {
            GetBundleByIdError::Backend => AppError::internal_server_error("internal server error"),
        })?
        .ok_or_else(|| AppError::NotFound)?
        .id;

    // find associated assets
    Ok(Json(
        state
            .repository
            .get_assets_by_bundle_id(id)
            .await
            .map_err(|err| match err {
                GetAssetsByBundleIdError::Backend => {
                    AppError::internal_server_error("internal server error")
                }
            })?
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
    use database::{CreateBundle, CreateBundleError};

    state
        .repository
        .create_bundle(&CreateBundle {
            created_by: auth.user().id,
        })
        .await
        .map(|bundle| Json(AssetBundleDto { id: bundle.id }))
        .map_err(|err| match err {
            CreateBundleError::Backend | CreateBundleError::Relationship => {
                AppError::internal_server_error("internal server error")
            }
        })
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
    use database::DeleteBundleError;

    state
        .repository
        .delete_bundle_by_id(id)
        .await
        .map_err(|err| match err {
            DeleteBundleError::Backend => AppError::internal_server_error("internal server error"),
            DeleteBundleError::Relationship => AppError::conflict("unable to delete bundle"),
            DeleteBundleError::NotFound => AppError::NotFound,
        })?;

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
    use database::ListBundlesError;

    Ok(Json(
        state
            .repository
            .list_bundles()
            .await
            .map_err(|err| match err {
                ListBundlesError::Backend => {
                    AppError::internal_server_error("internal server error")
                }
            })?
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
