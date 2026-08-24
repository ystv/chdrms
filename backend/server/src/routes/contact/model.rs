use serde::Serialize;
use url::Url;
use utoipa::ToSchema;

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
