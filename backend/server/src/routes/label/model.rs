use chdrms_database::label as database;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LabelDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub blocking: bool,
}

impl From<database::Label> for LabelDto {
    fn from(label: database::Label) -> Self {
        Self {
            id: label.id,
            name: label.name,
            description: label.description,
            colour: None, // todo: figure out what you're doing with this
            blocking: label.blocking,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LabelSummary {
    pub id: Uuid,
    pub name: String,
    pub colour: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub blocking: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateLabelRequest {
    pub name: String,
    pub description: Option<String>,
    pub colour: Option<String>,
    pub blocking: bool,
}
