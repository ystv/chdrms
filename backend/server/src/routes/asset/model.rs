use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AssetDto {
    /// The unique identifier for this asset. This property is immutable.
    pub id: Uuid,
    /// The type of this asset, as an asset type identifier. This property
    /// is immutable.
    pub r#type: Uuid,
    /// An optional alias for this asset. This is purely for convenience
    /// in searching for and identifying multiple instances of similar
    /// assets.
    pub alias: Option<String>,
    /// An optional field for miscellaneous notes about the asset.
    pub notes: Option<String>,
    /// The asset tag attached to this asset.
    pub tag: String,

    /// An optional bundle that this asset is within, as asset bundle
    /// identifier.
    pub bundle: Option<Uuid>,

    pub locations: AssetLocations,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateAssetLocationRequest {
    pub location: Uuid,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateAssetRequest {
    pub r#type: Uuid,
    pub alias: Option<String>,
    pub notes: Option<String>,
    pub tag: String,

    pub bundle: Option<Uuid>,

    pub locations: AssetLocations,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateAssetRequest {
    pub alias: Option<String>,
    pub notes: Option<String>,
    pub tag: String,

    pub bundle: Option<Uuid>,

    pub locations: UpdateAssetLocations,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AssetLocations {
    /// The current location of an asset. This property is only mutable
    /// via its dedicated field endpoints.
    pub current: Uuid,
    /// The default/home location of this asset. This is used to mark where
    /// an asset *should* be when not assigned to a project.
    pub home: Uuid,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateAssetLocations {
    pub home: Uuid,
}
