use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::{error::ErrorResponse, state::AppState};

pub mod auth;
pub mod group;
pub mod manufacturer;
pub mod user;

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/api/v1")
    ),
    tags(
        (name = user::TAG, description = "Users"),
        (name = auth::TAG, description = "Auth"),
        (name = group::TAG, description = "Groups"),
        (name = manufacturer::TAG, description = "Manufacturers"),
    ),
)]
struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    // TODO: check database connectivity
    "ok"
}

pub fn routes() -> (Router<AppState>, utoipa::openapi::OpenApi) {
    let (v1, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .nest("/auth", auth::api_routes())
        .nest("/group", group::routes())
        .nest("/user", user::routes())
        .nest("/manufacturer", manufacturer::routes())
        .split_for_parts();

    let router = Router::new()
        .nest("/v1", v1)
        .fallback(|| async { ErrorResponse::not_found() });

    (
        Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api.clone()))
            .nest("/api", router)
            .nest("/auth", auth::routes()),
        api,
    )
}
