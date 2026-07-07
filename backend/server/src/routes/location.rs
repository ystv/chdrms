use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::{PatchField, location as database};
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgPoint;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    state::AppState,
};

pub(super) const TAG: &str = "location";

#[derive(Serialize, ToSchema)]
struct LocationDto {
    id: Uuid,
    name: String,
    description: Option<String>,

    coordinates: Option<Coordinates>,
}

impl From<database::Location> for LocationDto {
    fn from(location: database::Location) -> Self {
        Self {
            id: location.id,
            name: location.name,
            description: location.description,

            coordinates: location.coordinates.map(Into::into),
        }
    }
}

#[derive(Deserialize, ToSchema)]
struct Location {
    name: String,
    description: Option<String>,

    coordinates: Option<Coordinates>,
}

impl Location {
    fn into_create(self, created_by: Uuid) -> database::CreateLocation {
        database::CreateLocation {
            name: self.name,
            description: self.description,

            coordinates: self.coordinates.map(Into::into),

            created_by,
        }
    }

    fn into_update(self) -> database::UpdateLocation {
        database::UpdateLocation {
            name: self.name,
            description: self.description,

            coordinates: self.coordinates.map(Into::into),
        }
    }
}

#[derive(Deserialize, ToSchema)]
struct PatchLocation {
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    name: PatchField<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>)]
    description: PatchField<Option<String>>,

    #[serde(default)]
    #[schema(value_type = Option<Coordinates>)]
    coordinates: PatchField<Option<Coordinates>>,
}

impl From<PatchLocation> for database::PatchLocation {
    fn from(location: PatchLocation) -> Self {
        Self {
            name: location.name,
            description: location.description,

            coordinates: location.coordinates.flat_map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct Coordinates(f64, f64);

impl From<Coordinates> for PgPoint {
    fn from(Coordinates(latitude, longitude): Coordinates) -> Self {
        Self {
            x: latitude,
            y: longitude,
        }
    }
}

impl From<PgPoint> for Coordinates {
    fn from(
        PgPoint {
            x: latitude,
            y: longitude,
        }: PgPoint,
    ) -> Self {
        Self(latitude, longitude)
    }
}

/// Get a location by its ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_location_by_id",
    responses(
        (status = OK, description = "Success", body = LocationDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Location by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<LocationDto>> {
    Ok(Json(
        database::Location::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .into(),
    ))
}

/// List all locations.
#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "list_locations",
    responses(
        (status = OK, description = "Success", body = [LocationDto]),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse)
    ),
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
) -> Result<Json<Vec<LocationDto>>> {
    Ok(Json(
        database::Location::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

/// Create a location.
#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    operation_id = "create_location",
    responses(
        (status = OK, description = "Success", body = LocationDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    auth: AuthContext,
    Json(create): Json<Location>,
) -> Result<Json<LocationDto>> {
    let mut txn = state.transaction().await?;
    let location = database::Location::create(&mut txn, create.into_create(auth.user().id)).await?;
    txn.commit().await?;

    Ok(Json(location.into()))
}

/// Delete a location by its ID.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    operation_id = "delete_location_by_id",
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Location by that ID not found", body = ErrorResponse),
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;
    database::Location::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;
    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update all fields of a location by its ID.
#[utoipa::path(
    put,
    path = "/{id}",
    tag = TAG,
    operation_id = "update_location_by_id",
    responses(
        (status = OK, description = "Success", body = LocationDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Location by that ID not found", body = ErrorResponse)
    ),
)]
async fn update(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(location): Json<Location>,
) -> Result<Json<LocationDto>> {
    let mut txn = state.transaction().await?;
    let location = database::Location::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .update(&mut txn, location.into_update())
        .await?;
    txn.commit().await?;

    Ok(Json(location.into()))
}

/// Patch individual fields of a location by its ID.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = TAG,
    operation_id = "patch_location_by_id",
    responses(
        (status = OK, description = "Success", body = LocationDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Location by that ID not found", body = ErrorResponse)
    )
)]
async fn patch(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(location): Json<PatchLocation>,
) -> Result<Json<LocationDto>> {
    let mut txn = state.transaction().await?;
    let location = database::Location::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .patch(&mut txn, location.into())
        .await?;
    txn.commit().await?;

    Ok(Json(location.into()))
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

#[cfg(test)]
mod tests {
    use crate::routes::location::Coordinates;

    #[test]
    fn test_coordinates_to_json() {
        let coordinates = Coordinates(69., 42.);
        assert_eq!(
            serde_json::to_string(&coordinates).expect("failed to serialize coordinates"),
            "[69.0,42.0]",
        );
    }

    #[test]
    fn test_coordinates_from_json() {
        let Coordinates(x, y) =
            serde_json::from_str("[42.0,69.0]").expect("failed to deserialize coordinates");
        assert_eq!(x, 42.);
        assert_eq!(y, 69.);
    }
}
