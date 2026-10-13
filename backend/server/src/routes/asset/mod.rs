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
    asset::{self as database, AssetRepository, CreateAsset, UpdateAsset},
    location::LocationRepository,
};

use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::{
        asset::model::{
            AssetDto, AssetLocations, CreateAssetRequest, UpdateAssetLocationRequest,
            UpdateAssetRequest,
        },
        location::LocationDto,
        model::AssetIdentifier,
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
    use database::GetAssetByIdError;

    match id {
        AssetIdentifier::Id(id) => {
            state
                .repository
                .get_asset_by_id(id)
                .await
                .map_err(|err| match err {
                    GetAssetByIdError::Backend => {
                        AppError::internal_server_error("internal server error")
                    }
                })?
        }
        AssetIdentifier::Tag(tag) => {
            database::Asset::get_by_tag(&mut state.transaction().await?, tag).await?
        }
    }
    .ok_or_else(|| AppError::NotFound)
    .map(|asset| {
        Json(AssetDto {
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
    })
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
    use database::CreateAssetError;

    state
        .repository
        .create_asset(&CreateAsset {
            r#type: asset.r#type,
            alias: asset.alias,
            notes: asset.notes,
            tag: asset.tag,
            bundle: asset.bundle,
            home_location: asset.locations.home,
            location: asset.locations.current,
            created_by: auth.user().id,
        })
        .await
        .map_err(|err| match err {
            CreateAssetError::Backend | CreateAssetError::Relationship => {
                AppError::internal_server_error("internal server error")
            }
        })
        .map(|asset| {
            Json(AssetDto {
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
        })
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
    use database::{GetAssetByIdError, UpdateAssetByIdError};

    // get location from existing asset
    // todo: remove location from the update struct so that
    //       this extra lookup isn't required.
    let asset = state
        .repository
        .get_asset_by_id(id)
        .await
        .map_err(|err| match err {
            GetAssetByIdError::Backend => AppError::internal_server_error("internal server error"),
        })?
        .ok_or_else(|| AppError::NotFound)?;
    let location = asset.location;

    // update asset
    state
        .repository
        .update_asset_by_id(
            id,
            &UpdateAsset {
                alias: update.alias,
                notes: update.notes,
                tag: update.tag,
                bundle: update.bundle,
                home_location: update.locations.home,
                location,
            },
        )
        .await
        .map_err(|err| match err {
            UpdateAssetByIdError::Backend => {
                AppError::internal_server_error("internal server error")
            }
            UpdateAssetByIdError::Relationship => AppError::bad_request("invalid entity"),
            UpdateAssetByIdError::NotFound => AppError::NotFound,
        })
        .map(|asset| {
            Json(AssetDto {
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
        })
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
    use database::ListAssetsError;

    Ok(Json(
        state
            .repository
            .list_assets()
            .await
            .map_err(|err| match err {
                ListAssetsError::Backend => {
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
    use database::DeleteAssetByIdError;

    state
        .repository
        .delete_asset_by_id(id)
        .await
        .map_err(|err| match err {
            DeleteAssetByIdError::Backend => {
                AppError::internal_server_error("internal server error")
            }
            DeleteAssetByIdError::Relationship => AppError::conflict("failed to delete asset"),
            DeleteAssetByIdError::NotFound => AppError::NotFound,
        })
        .map(|_| StatusCode::NO_CONTENT)
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
    use chdrms_database::location::GetLocationByAssetIdError;

    state
        .repository
        .get_location_by_asset_id(id)
        .await
        .map_err(|err| match err {
            GetLocationByAssetIdError::Backend => {
                AppError::internal_server_error("internal server error")
            }
            GetLocationByAssetIdError::NotFound => AppError::NotFound,
        })
        .map(|location| Json(location.into()))
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
    use database::{GetAssetByIdError, SetAssetLocationByIdError};

    // check if the asset exists
    let id = state
        .repository
        .get_asset_by_id(id)
        .await
        .map_err(|err| match err {
            GetAssetByIdError::Backend => AppError::internal_server_error("internal server error"),
        })?
        .ok_or_else(|| AppError::NotFound)?
        .id;

    state
        .repository
        .set_asset_location_by_id(id, location)
        .await
        .map_err(|err| match err {
            SetAssetLocationByIdError::Backend => {
                AppError::internal_server_error("internal server error")
            }
            SetAssetLocationByIdError::Relationship => AppError::bad_request("invalid location"),
            SetAssetLocationByIdError::NotFound => AppError::NotFound,
        })
        .map(|asset| {
            Json(AssetDto {
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
        })
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
}
