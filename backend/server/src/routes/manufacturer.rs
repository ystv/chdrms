use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{PatchField, manufacturer as database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    state::AppState,
};

pub(super) const TAG: &str = "manufacturer";

#[derive(Serialize, ToSchema)]
pub struct ManufacturerInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl From<database::Manufacturer> for ManufacturerInfo {
    fn from(manufacturer: database::Manufacturer) -> Self {
        Self {
            id: manufacturer.id,
            name: manufacturer.name,
            description: manufacturer.description,

            website: manufacturer.website,
            email: manufacturer.email,
            phone: manufacturer.phone,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct Manufacturer {
    name: String,
    description: Option<String>,

    website: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

impl Manufacturer {
    fn into_create(self, created_by: Uuid) -> database::CreateManufacturer {
        database::CreateManufacturer {
            name: self.name,
            description: self.description,
            website: self.website,
            email: self.email,
            phone: self.phone,
            created_by,
        }
    }

    fn into_update(self) -> database::UpdateManufacturer {
        database::UpdateManufacturer {
            name: self.name,
            description: self.description,
            website: self.website,
            email: self.email,
            phone: self.phone,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct PatchManufacturer {
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    name: PatchField<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    description: PatchField<Option<String>>,

    #[serde(default)]
    #[schema(value_type = Option<String>)]
    website: PatchField<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    email: PatchField<Option<String>>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    phone: PatchField<Option<String>>,
}

impl From<PatchManufacturer> for database::PatchManufacturer {
    fn from(manufacturer: PatchManufacturer) -> Self {
        Self {
            name: manufacturer.name,
            description: manufacturer.description,
            website: manufacturer.website,
            email: manufacturer.email,
            phone: manufacturer.phone,
        }
    }
}

/// Get a manufacturer by its ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_manufacturer_by_id",
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse),
    )
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<ManufacturerInfo>> {
    Ok(Json(
        database::Manufacturer::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .into(),
    ))
}

/// List all manufacturers.
#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "list_manufacturers",
    responses(
        (status = OK, description = "Success", body = [ManufacturerInfo]),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse)
    )
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
) -> Result<Json<Vec<ManufacturerInfo>>> {
    Ok(Json(
        database::Manufacturer::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

/// Create a manufacturer.
#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    operation_id = "create_manufacturer",
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse)
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    auth: AuthContext,
    Json(create): Json<Manufacturer>,
) -> Result<Json<ManufacturerInfo>> {
    let mut txn = state.transaction().await?;
    let manufacturer =
        database::Manufacturer::create(&mut txn, create.into_create(auth.user().id)).await?;
    txn.commit().await?;

    Ok(Json(manufacturer.into()))
}

/// Delete a manufacturer by its ID.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    operation_id = "delete_manufacturer_by_id",
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse)
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    database::Manufacturer::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update all fields of a manufacturer by its ID.
#[utoipa::path(
    put,
    path = "/{id}",
    tag = TAG,
    operation_id = "update_manufacturer_by_id",
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse)
    ),
)]
async fn update(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(manufacturer): Json<Manufacturer>,
) -> Result<Json<ManufacturerInfo>> {
    let mut txn = state.transaction().await?;
    let manufacturer = database::Manufacturer::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .update(&mut txn, manufacturer.into_update())
        .await?;
    txn.commit().await?;

    Ok(Json(manufacturer.into()))
}

/// Patch individual fields of a manufacturer by its ID.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = TAG,
    operation_id = "patch_manufacturer_by_id",
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse)
    )
)]
async fn patch(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(manufacturer): Json<PatchManufacturer>,
) -> Result<Json<ManufacturerInfo>> {
    let mut txn = state.transaction().await?;
    let manufacturer = database::Manufacturer::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .patch(&mut txn, manufacturer.into())
        .await?;
    txn.commit().await?;

    Ok(Json(manufacturer.into()))
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id))
        .routes(routes!(list, create))
        .routes(routes!(delete))
        .routes(routes!(update))
        .routes(routes!(patch))
}
