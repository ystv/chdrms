use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
};
use axum_extra::extract::{CookieJar, PrivateCookieJar, cookie::Cookie};
use chdrms_database::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{SESSION_COOKIE, oidc::OIDCProvider},
    error::{AppError, Result},
    state::AppState,
};

pub(super) const TAG: &str = "auth";

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

async fn logout(jar: CookieJar, State(state): State<AppState>) -> Result<(CookieJar, Redirect)> {
    let mut txn = state.transaction().await?;

    let cookie = jar.get(SESSION_COOKIE).ok_or(AppError::Unauthorized)?;

    let Ok(token) = Uuid::parse_str(cookie.value()) else {
        return Err(AppError::Unauthorized);
    };

    let jar = jar.remove({
        let mut cookie = Cookie::from(SESSION_COOKIE);
        cookie.set_path("/");
        cookie
    });

    if !User::destroy_session(&mut txn, token).await? {
        return Err(AppError::Unauthorized);
    }

    Ok((jar, Redirect::to("/")))
}

/// List authentication providers.
#[utoipa::path(
    method(get),
    path = "/providers",
    tag = TAG,
    operation_id = "list_auth_providers",
    responses(
        (status = OK, description = "Success", body = [ProviderInfo]),
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
        .route("/logout", post(logout))
}

pub fn api_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(list_providers))
}
