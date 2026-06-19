use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::manufacturer::{self, Manufacturer};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::permissions::RequirePermission,
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

impl From<&Manufacturer> for ManufacturerInfo {
    fn from(manufacturer: &Manufacturer) -> Self {
        Self {
            id: manufacturer.id,
            name: manufacturer.name.clone(),
            description: manufacturer.description.clone(),

            website: manufacturer.website.clone(),
            email: manufacturer.email.clone(),
            phone: manufacturer.phone.clone(),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateManufacturer {
    name: String,
    description: Option<String>,

    website: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

impl From<&CreateManufacturer> for chdrms_database::manufacturer::ManufacturerCreation {
    fn from(manufacturer: &CreateManufacturer) -> Self {
        Self {
            name: manufacturer.name.clone(),
            description: manufacturer.description.clone(),
            website: manufacturer.website.clone(),
            email: manufacturer.email.clone(),
            phone: manufacturer.phone.clone(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
    )
)]
async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    _auth: RequirePermission<manufacturer::permission::View>,
) -> Result<Json<ManufacturerInfo>> {
    Ok(Json(
        (&Manufacturer::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?)
            .into(),
    ))
}

#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = [ManufacturerInfo]),
    )
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<manufacturer::permission::View>,
) -> Result<Json<Vec<ManufacturerInfo>>> {
    Ok(Json(
        Manufacturer::list(&mut state.transaction().await?)
            .await?
            .iter()
            .map(From::from)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    responses(
        (status = OK, description = "Success", body = ManufacturerInfo),
    ),
)]
pub async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<manufacturer::permission::Manage>,
    Json(create): Json<CreateManufacturer>,
) -> Result<Json<ManufacturerInfo>> {
    let mut txn = state.transaction().await?;
    let manufacturer = Manufacturer::create(&mut txn, (&create).into()).await?;
    txn.commit().await?;

    Ok(Json((&manufacturer).into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Manufacturer by that ID not found", body = ErrorResponse)
    ),
)]
pub async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<manufacturer::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    Manufacturer::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id))
        .routes(routes!(list, create))
        .routes(routes!(delete))
}
