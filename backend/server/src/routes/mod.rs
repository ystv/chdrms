use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    // TODO: check database connectivity
    "ok"
}

pub fn routes(state: AppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .with_state(state)
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    router
}
