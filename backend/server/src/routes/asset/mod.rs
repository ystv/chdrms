use axum::{
    Json,
    extract::{Path, Query, State},
};
use chdrms_database::asset as database;

use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::permissions::RequirePermission,
    error::{AppError, ErrorResponse, Result},
    routes::asset::model::{AssetDto, AssetSummary, ResolvableBundle},
    state::AppState,
};

mod model;

pub(super) const TAG: &str = "asset";

/// Get an asset by its ID.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = TAG,
    operation_id = "get_asset_by_id",
    params(
        ("id" = Uuid, Path, description = "Requested asset ID"),
        ("resolve" = Option<bool>, Query, description = "Whether to resolve related entites")
    ),
    responses(
        (status = OK, description = "Success", body = AssetDto),
        (status = UNAUTHORIZED, description = "Missing permission", body = ErrorResponse),
        (status = NOT_FOUND, description = "Asset by that ID not found", body = ErrorResponse)
    ),
)]
async fn get_by_id(
    State(state): State<AppState>,
    _auth: RequirePermission<database::permission::View>,
    Path(id): Path<Uuid>,
    Query(resolve): Query<Option<bool>>,
) -> Result<Json<AssetDto>> {
    let mut txn = state.transaction().await?;
    let asset = database::Asset::get_by_id(&mut txn, id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    let bundle = match (resolve.unwrap_or_default(), asset.bundle) {
        (true, Some(bundle)) => match asset.get_bundle_partner_summaries(&mut txn).await {
            Some(Ok(assets)) => {
                let assets = assets
                    .into_iter()
                    .map(|asset| AssetSummary {
                        id: asset.id,
                        alias: asset.alias,
                        tag: asset.tag,
                    })
                    .collect();
                Some(ResolvableBundle::Resolved { id: bundle, assets })
            }
            Some(Err(err)) => return Err(err.into()),
            _ => None,
        },
        (false, Some(bundle)) => Some(ResolvableBundle::Unresolved(bundle)),
        (_, None) => None,
    };

    Ok(Json(AssetDto {
        id,
        r#type: asset.r#type,
        alias: asset.alias,
        tag: asset.tag,
        bundle,
        home_location: asset.home_location,
        location: asset.location,
    }))
}

pub(super) fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_by_id))
}
