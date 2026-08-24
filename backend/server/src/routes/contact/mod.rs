use axum::{
    Json,
    extract::{Query, State},
};
use chdrms_database::asset::Asset;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    config::ContactLink,
    error::{AppError, ErrorResponse, Result},
    routes::contact::model::{ContactDetailsDto, ContactDetailsLinkDto, ContactParameters},
    state::AppState,
};

mod model;

pub(super) const TAG: &str = "contact";

/// Get this instance's contact details.
#[utoipa::path(
    get,
    path = "/",
    tag = TAG,
    operation_id = "get_contact_details",
    params(
        ("asset", Query, description = "Referred asset ID or Tag."),
    ),
    responses(
        (status = OK, description = "Success", body = ContactDetailsDto),
        (status = FORBIDDEN, description = "Forbidden", body = ErrorResponse),
    ),
)]
async fn get_contact_details(
    State(state): State<AppState>,
    Query(parameters): Query<ContactParameters>,
) -> Result<Json<ContactDetailsDto>> {
    if !state.config.contact_details.always_show {
        let Some(asset) = parameters.asset else {
            return Err(AppError::forbidden("forbidden"));
        };

        if !Asset::asset_exists(&mut state.transaction().await?, &asset.into()).await? {
            return Err(AppError::forbidden("forbidden"));
        }
    }

    let details = state.config.contact_details;

    Ok(Json(ContactDetailsDto {
        name: details.name,
        links: details
            .links
            .into_iter()
            .map(|ContactLink { label, link }| ContactDetailsLinkDto { label, link })
            .collect(),
    }))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_contact_details))
}
