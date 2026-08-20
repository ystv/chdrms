use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use chdrms_database::Postgres;
use openidconnect::reqwest;

use crate::{USER_AGENT, config::AppConfig};

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
    pub repository: Postgres,
    pub config: AppConfig,
    pub client: reqwest::Client,
    pub key: Key,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool, repository: Postgres, config: AppConfig, key: Key) -> Self {
        Self {
            pool,
            repository,
            config,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            key,
        }
    }

    #[deprecated]
    pub async fn transaction(&self) -> sqlx::Result<sqlx::PgTransaction<'_>> {
        self.pool.begin().await
    }
}
