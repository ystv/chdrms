use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub(super) struct AssetDto {
    pub id: Uuid,
    pub r#type: Uuid,
    pub alias: Option<String>,
    pub tag: String,

    pub bundle: Option<ResolvableBundle>,

    pub home_location: Uuid,
    pub location: Uuid,
}

#[derive(Serialize, ToSchema)]
pub(super) struct AssetSummary {
    pub id: Uuid,
    pub alias: Option<String>,
    pub tag: String,
}

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub(super) enum ResolvableBundle {
    Unresolved(Uuid),
    Resolved { id: Uuid, assets: Vec<AssetSummary> },
}
