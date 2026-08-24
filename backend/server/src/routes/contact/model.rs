use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::routes::model::AssetIdentifier;

#[derive(Serialize, ToSchema)]
pub(super) struct ContactDetailsDto {
    pub name: Option<String>,
    pub links: Vec<ContactDetailsLinkDto>,
}

#[derive(Serialize, ToSchema)]
pub(super) struct ContactDetailsLinkDto {
    pub label: Option<String>,
    pub link: Url,
}

#[derive(Deserialize)]
pub(super) struct ContactParameters {
    pub asset: Option<AssetIdentifier>,
}
