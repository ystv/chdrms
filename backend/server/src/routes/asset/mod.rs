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
    asset::{self as database, AssetId, CreateAsset, UpdateAsset},
    asset_type::AssetTypeId,
    bundle::BundleId,
    location::LocationId,
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
    let mut txn = state.transaction().await?;
    let asset = match id {
        AssetIdentifier::Id(id) => database::Asset::get_by_id(&mut txn, AssetId::new(id)).await,
        AssetIdentifier::Tag(tag) => database::Asset::get_by_tag(&mut txn, tag).await,
    }?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(AssetDto {
        id: asset.id.into(),
        r#type: asset.r#type.into(),
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle.map(Into::into),
        locations: AssetLocations {
            current: asset.location.into(),
            home: asset.home_location.into(),
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
            // todo:
            // at the moment, we trust the client has provided us with valid IDs.
            // however, we should probably do lookups so that the error provided
            // back to the client is more precise.
            r#type: AssetTypeId::new(asset.r#type),
            alias: asset.alias,
            notes: asset.notes,
            tag: asset.tag,
            bundle: asset.bundle.map(BundleId::new),
            home_location: LocationId::new(asset.locations.home),
            location: LocationId::new(asset.locations.current),
            created_by: auth.user().id,
        },
    )
    .await?;

    txn.commit().await?;

    Ok(Json(AssetDto {
        id: asset.id.into(),
        r#type: asset.r#type.into(),
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle.map(Into::into),
        locations: AssetLocations {
            current: asset.location.into(),
            home: asset.home_location.into(),
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

    let asset = database::Asset::get_by_id(&mut txn, AssetId::new(id))
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
                // todo: we should verify these IDs are valid first.
                bundle: update.bundle.map(BundleId::new),
                home_location: LocationId::new(update.locations.home),
                location,
            },
        )
        .await?;

    Ok(Json(AssetDto {
        id: asset.id.into(),
        r#type: asset.r#type.into(),
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle.map(Into::into),
        locations: AssetLocations {
            current: asset.location.into(),
            home: asset.home_location.into(),
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
                id: asset.id.into(),
                r#type: asset.r#type.into(),
                alias: asset.alias,
                notes: asset.notes,
                tag: asset.tag,
                bundle: asset.bundle.map(Into::into),
                locations: AssetLocations {
                    current: asset.location.into(),
                    home: asset.home_location.into(),
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
    let mut txn = state.transaction().await?;

    database::Asset::get_by_id(&mut txn, AssetId::new(id))
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
        database::Asset::get_by_id(&mut txn, AssetId::new(id))
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

    let asset = database::Asset::get_by_id(&mut txn, AssetId::new(id))
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .set_location(&mut txn, LocationId::new(location)) // todo: we should check the existence of the location beforehand
        .await?;

    txn.commit().await?;

    Ok(Json(AssetDto {
        id: asset.id.into(),
        r#type: asset.r#type.into(),
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        bundle: asset.bundle.map(Into::into),
        locations: AssetLocations {
            current: asset.location.into(),
            home: asset.home_location.into(),
        },
    }))
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
