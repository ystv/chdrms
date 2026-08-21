use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chdrms_database::label::{self as database, CreateLabel, UpdateLabel};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthContext, permissions::RequirePermission},
    error::{AppError, ErrorResponse, Result},
    routes::label::model::{CreateLabelRequest, LabelDto, UpdateLabelRequest},
    state::AppState,
};

pub(super) mod model;

pub(super) const TAG: &str = "label";

#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_label_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested label ID."),
    ),
    responses(
        (status = OK, description = "Success", body = LabelDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Label by that ID not found", body = ErrorResponse),
    ),
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
) -> Result<Json<LabelDto>> {
    Ok(Json(
        database::Label::get_by_id(&mut state.transaction().await?, id)
            .await?
            .ok_or_else(|| AppError::NotFound)?
            .into(),
    ))
}

#[utoipa::path(
    post,
    path = "/",
    tag = TAG,
    operation_id = "create_label",
    responses(
        (status = OK, description = "Success", body = LabelDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn create(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    auth: AuthContext,
    Json(request): Json<CreateLabelRequest>,
) -> Result<Json<LabelDto>> {
    let mut txn = state.transaction().await?;

    let label = database::Label::create(
        &mut txn,
        CreateLabel {
            name: request.name,
            description: request.description,
            colour: None, // todo: figure out how to parse this
            blocking: request.blocking,
            created_by: auth.user().id,
        },
    )
    .await?;

    txn.commit().await?;

    Ok(Json(label.into()))
}

#[utoipa::path(
    put,
    path = "/{id}",
    tag = TAG,
    operation_id = "update_label_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested label ID."),
    ),
    responses(
        (status = OK, description = "Success", body = LabelDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Label by that ID not found", body = ErrorResponse),
    ),
)]
async fn update(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
    Json(update): Json<UpdateLabelRequest>,
) -> Result<Json<LabelDto>> {
    let mut txn = state.transaction().await?;

    let label = database::Label::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .update(
            &mut txn,
            UpdateLabel {
                name: update.name,
                description: update.description,
                colour: None, // todo: figure out how to parse this
                blocking: update.blocking,
            },
        )
        .await?;

    txn.commit().await?;

    Ok(Json(label.into()))
}

#[utoipa::path(
    delete,
    path = "/{id}",
    tag = TAG,
    operation_id = "delete_label_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested label ID."),
    ),
    responses(
        (status = NO_CONTENT, description = "Success"),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Label by that ID not found", body = ErrorResponse),
    ),
)]
async fn delete(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::Manage>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let mut txn = state.transaction().await?;

    database::Label::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?
        .delete(&mut txn)
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "list_labels",
    responses(
        (status = OK, description = "Success", body = Vec<LabelDto>),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
    ),
)]
async fn list(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
) -> Result<Json<Vec<LabelDto>>> {
    Ok(Json(
        database::Label::list(&mut state.transaction().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_by_id))
        .routes(routes!(create))
        .routes(routes!(update))
        .routes(routes!(delete))
        .routes(routes!(list))
}
