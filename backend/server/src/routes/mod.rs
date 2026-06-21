use axum::Router;
use serde::{Deserialize, Deserializer};
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

#[derive(Debug, PartialEq, Eq)]
pub enum PatchField<T> {
    Present(T),
    Absent,
}

impl<T> PatchField<T> {
    pub fn is_absent(&self) -> bool {
        matches!(self, PatchField::Absent)
    }
}

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Absent
    }
}

impl<'de, T> Deserialize<'de> for PatchField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(PatchField::Present(T::deserialize(deserializer)?))
    }
}

impl<T: Clone> From<&PatchField<T>> for chdrms_database::PatchField<T> {
    fn from(field: &PatchField<T>) -> Self {
        match field {
            PatchField::Present(value) => chdrms_database::PatchField::Present(value.clone()),
            PatchField::Absent => chdrms_database::PatchField::Absent,
        }
    }
}

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
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TestPatch {
        #[serde(default)]
        field: PatchField<String>,
        #[serde(default)]
        nullable_field: PatchField<Option<String>>,
    }

    #[test]
    fn absent_when_missing() {
        let patch: TestPatch = serde_json::from_str("{}").unwrap();
        assert!(patch.field.is_absent());
        assert!(patch.nullable_field.is_absent());
    }

    #[test]
    fn present_some_when_value_given() {
        let patch: TestPatch = serde_json::from_str(r#"{"field": "Hello, World!"}"#).unwrap();
        assert_eq!(
            patch.field,
            PatchField::Present("Hello, World!".to_string())
        );
        assert!(patch.nullable_field.is_absent());
    }

    #[test]
    fn present_none_when_null_given() {
        let patch: TestPatch = serde_json::from_str(r#"{"nullable_field": null}"#).unwrap();
        assert!(patch.field.is_absent());
        assert_eq!(patch.nullable_field, PatchField::Present(None));
    }

    #[test]
    fn non_nullable_rejects_null() {
        let result: Result<TestPatch, _> = serde_json::from_str(r#"{"field": null}"#);
        assert!(result.is_err());
    }
}
