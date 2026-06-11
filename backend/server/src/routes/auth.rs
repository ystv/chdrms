use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::get,
};
use axum_extra::extract::{CookieJar, PrivateCookieJar};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::oidc::OIDCProvider,
    error::{AppError, Result},
    state::AppState,
};

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

#[derive(Serialize, ToSchema)]
struct ProviderInfo {
    id: String,
    name: String,
}

async fn get_provider(state: &AppState, name: &str) -> Result<OIDCProvider> {
    if let Some(provider_config) = state.config.oidc_providers.get(name) {
        Ok(OIDCProvider::new(
            &state.config.base_url,
            name,
            provider_config.clone(),
            state.client.clone(),
        )
        .await?)
    } else {
        Err(AppError::NotFound)
    }
}

async fn begin(
    Path(provider): Path<String>,
    jar: PrivateCookieJar,
    State(state): State<AppState>,
) -> Result<(PrivateCookieJar, Redirect)> {
    Ok(get_provider(&state, &provider).await?.prepare_auth(jar))
}

async fn callback(
    Path(provider): Path<String>,
    jar: CookieJar,
    private_jar: PrivateCookieJar,
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> Result<(PrivateCookieJar, CookieJar, Redirect)> {
    let provider = get_provider(&state, &provider).await?;
    let mut txn = state.transaction().await?;
    let res = provider
        .complete_auth(&mut txn, private_jar, jar, query.code, query.state)
        .await
        .map_err(Into::<AppError>::into)?;
    txn.commit().await?;
    Ok(res)
}

#[utoipa::path(
    method(get),
    path = "/providers",
    responses(
        (status = OK, description = "Success", body = [Vec<ProviderInfo>]),
    ),
)]
async fn list_providers(State(state): State<AppState>) -> Json<Vec<ProviderInfo>> {
    Json(
        state
            .config
            .oidc_providers
            .iter()
            .map(|(id, p)| ProviderInfo {
                id: id.clone(),
                name: p.name.clone(),
            })
            .collect(),
    )
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{provider}/begin", get(begin))
        .route("/{provider}/callback", get(callback))
}

pub fn api_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_providers))
}
