use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
            AssetIdentifier::Id(id) => Self::Id(id),
            AssetIdentifier::Tag(tag) => Self::Tag(tag),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentDto {
    pub id: Uuid,
    pub archived_at: Option<DateTime<Utc>>,

    pub title: String,
    pub content: String,

    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateCommentRequest {
    pub archived: bool,
    pub title: String,
    pub content: String,
}
