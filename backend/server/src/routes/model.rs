use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use utoipa::ToSchema;
use uuid::Uuid;

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
