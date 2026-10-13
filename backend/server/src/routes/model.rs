use chdrms_database::asset::AssetId;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[serde(untagged)]
pub enum AssetIdentifier {
    Id(Uuid),
    Tag(String),
}

impl From<AssetIdentifier> for chdrms_database::asset::AssetIdentifier {
    fn from(identifier: AssetIdentifier) -> Self {
        match identifier {
            AssetIdentifier::Id(id) => Self::Id(AssetId::new(id)), // todo: this probably isn't safe
            AssetIdentifier::Tag(tag) => Self::Tag(tag),
        }
    }
}
