use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    config::ContactLink,
    routes::contact::model::{ContactDetailsDto, ContactDetailsLinkDto},
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
    responses(
        (status = OK, description = "Success", body = ContactDetailsDto),
    ),
)]
async fn get_contact_details(State(state): State<AppState>) -> Json<ContactDetailsDto> {
    let details = state.config.contact_details;
    Json(ContactDetailsDto {
        name: details.name,
        links: details
            .links
            .into_iter()
            .map(|ContactLink { label, link }| ContactDetailsLinkDto { label, link })
            .collect(),
    })
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_contact_details))
}
