use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use openidconnect::reqwest;

use crate::{USER_AGENT, config::AppConfig};

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
    pub config: AppConfig,
    pub client: reqwest::Client,
    pub key: Key,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, config: AppConfig, key: Key) -> Self {
        Self {
            pool,
            config,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            key,
        }
    }

    pub async fn transaction(&self) -> sqlx::Result<sqlx::PgTransaction<'_>> {
        self.pool.begin().await
    }
}
