use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Text;
use url::Url;
use utoipa::{PartialSchema, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use chdrms_database::{PatchField, asset_type as database, manufacturer::Manufacturer};

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    state::AppState,
};

pub(super) const TAG: &str = "asset_type";

#[derive(Serialize, ToSchema)]
struct AssetTypeDto {
    id: Uuid,
    name: String,
    manufacturer: Uuid,

    product_url: Option<Url>,
    value: Option<SchemaDecimal>,
}

impl From<database::AssetType> for AssetTypeDto {
    fn from(asset_type: database::AssetType) -> Self {
        Self {
            id: asset_type.id,
            name: asset_type.name,
            manufacturer: asset_type.manufacturer,
            product_url: asset_type.product_url.map(|url| url.0),
            value: asset_type.value.map(Into::into),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct SchemaDecimal(Decimal);

impl From<SchemaDecimal> for Decimal {
    fn from(decimal: SchemaDecimal) -> Self {
        decimal.0
    }
}

impl From<Decimal> for SchemaDecimal {
    fn from(decimal: Decimal) -> Self {
        SchemaDecimal(decimal)
    }
}

impl PartialSchema for SchemaDecimal {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        f32::schema()
    }
}

impl ToSchema for SchemaDecimal {}

#[derive(Deserialize, ToSchema)]
struct AssetType {
    name: String,
    manufacturer: Uuid,

    product_url: Option<Url>,
    value: Option<SchemaDecimal>,
}

impl AssetType {
    fn into_create(self, created_by: Uuid) -> database::CreateAssetType {
        database::CreateAssetType {
            name: self.name,
            manufacturer: self.manufacturer,
            product_url: self.product_url.map(Text),
            value: self.value.map(Into::into),
            created_by,
        }
    }

    fn into_update(self) -> database::UpdateAssetType {
        database::UpdateAssetType {
            name: self.name,
            manufacturer: self.manufacturer,
            product_url: self.product_url.map(Text),
            value: self.value.map(Into::into),
        }
    }
}

#[derive(Deserialize, ToSchema)]
struct PatchAssetType {
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    name: PatchField<String>,
    #[serde(default)]
    #[schema(value_type = Option<Uuid>)]
    manufacturer: PatchField<Uuid>,

    #[serde(default)]
    #[schema(value_type = Option<String>)]
    product_url: PatchField<Option<Url>>,
    #[serde(default)]
    #[schema(value_type = Option<SchemaDecimal>)]
    value: PatchField<Option<Decimal>>,
}

impl From<PatchAssetType> for database::PatchAssetType {
    fn from(asset_type: PatchAssetType) -> Self {
        Self {
            name: asset_type.name,
            manufacturer: asset_type.manufacturer,
            product_url: asset_type.product_url.flat_map(Text),
            value: asset_type.value,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = AssetTypeDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description= "Asset type by that ID not found", body = ErrorResponse),
    )
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssetTypeDto>> {
    Ok(Json(
        database::AssetType::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .into(),
    ))
}

#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [AssetTypeDto]),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
) -> Result<Json<Vec<AssetTypeDto>>> {
    Ok(Json(
        database::AssetType::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = AssetTypeDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = BAD_REQUEST, description = "Manufacturer by that ID not found", body = ErrorResponse),
    )
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    auth: AuthContext,
    Json(create): Json<AssetType>,
) -> Result<Json<AssetTypeDto>> {
    let mut txn = state.transaction().await?;
    if let None = Manufacturer::get_by_id(&mut txn, create.manufacturer).await? {
        return Err(AppError::bad_request("manufacturer not found"));
    }
    let asset_type =
        database::AssetType::create(&mut txn, create.into_create(auth.user().id)).await?;
    txn.commit().await?;

    Ok(Json(asset_type.into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset type by that ID not found", body = ErrorResponse),
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    database::AssetType::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = AssetTypeDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset type by that ID not found", body = ErrorResponse),
    ),
)]
async fn update(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(asset_type): Json<AssetType>,
) -> Result<Json<AssetTypeDto>> {
    let mut txn = state.transaction().await?;
    let asset_type = database::AssetType::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .update(&mut txn, asset_type.into_update())
        .await?;
    txn.commit().await?;

    Ok(Json(asset_type.into()))
}

#[utoipa::path(
    patch,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = AssetTypeDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse)
    ),
)]
async fn patch(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(asset_type): Json<PatchAssetType>,
) -> Result<Json<AssetTypeDto>> {
    let mut txn = state.transaction().await?;
    let asset_type = database::AssetType::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .patch(&mut txn, asset_type.into())
        .await?;
    txn.commit().await?;

    Ok(Json(asset_type.into()))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id))
        .routes(routes!(list))
        .routes(routes!(create))
        .routes(routes!(delete))
        .routes(routes!(update))
        .routes(routes!(patch))
}
