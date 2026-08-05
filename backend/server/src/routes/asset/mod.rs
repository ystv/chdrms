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
    label::LabelAndBlocking,
};

use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::{
        asset::model::{
            AssetDto, AssetLocations, Block, CreateAssetRequest, UpdateAssetLocationRequest,
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
        AssetIdentifier::Id(id) => database::Asset::get_by_id(&mut txn, id).await,
        AssetIdentifier::Tag(tag) => database::Asset::get_by_tag(&mut txn, tag).await,
    }?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(populate_asset_dto(asset, &mut txn).await?))
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
        // we can assume there are no labels attach to this
        // asset as it has just been created.
        labels: Vec::new(),
        bundle: asset.bundle,
        // we can assume there are no blocks on this asset
        // as it has just been created.
        blocks: Vec::new(),
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

    Ok(Json(populate_asset_dto(asset, &mut txn).await?))
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
    let mut txn = state.transaction().await?;

    Ok(Json(
        populate_asset_dtos(database::Asset::list(&mut txn).await?, &mut txn).await?,
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

    let asset = populate_asset_dto(asset, &mut txn).await?;

    txn.commit().await?;

    Ok(Json(asset))
}

async fn populate_asset_dto(
    asset: database::Asset,
    txn: &mut sqlx::PgTransaction<'_>,
) -> sqlx::Result<AssetDto> {
    let labels =
        chdrms_database::label::Label::list_label_and_blocking_for_asset(txn, asset.id).await?;
    Ok(populate_asset_dto_raw(asset, labels))
}

pub(super) async fn populate_asset_dtos(
    assets: Vec<database::Asset>,
    txn: &mut sqlx::PgTransaction<'_>,
) -> sqlx::Result<Vec<AssetDto>> {
    // find labels
    let labels = chdrms_database::label::Label::list_label_and_blocking_for_assets(
        txn,
        assets.iter().map(|asset| asset.id).collect(),
    )
    .await?;

    Ok(assets
        .into_iter()
        .map(|asset| {
            let labels = labels.get(&asset.id).cloned().unwrap_or_default();
            populate_asset_dto_raw(asset, labels)
        })
        .collect())
}

fn populate_asset_dto_raw(asset: database::Asset, labels: Vec<LabelAndBlocking>) -> AssetDto {
    // construct blocks
    let blocks = labels
        .iter()
        .filter(|label| label.blocking)
        .map(|label| Block::Label { label: label.id })
        .collect();

    // construct labels
    let labels = labels.iter().map(|label| label.id).collect();

    AssetDto {
        id: asset.id,
        r#type: asset.r#type,
        alias: asset.alias,
        notes: asset.notes,
        tag: asset.tag,
        labels,
        bundle: asset.bundle,
        blocks,
        locations: AssetLocations {
            current: asset.location,
            home: asset.home_location,
        },
    }
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
