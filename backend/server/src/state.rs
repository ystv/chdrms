use axum::extract::FromRef;

#[derive(Clone, FromRef)]
pub struct AppState {
    pool: sqlx::PgPool,
}

impl AppState {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn transaction(&self) -> sqlx::Result<sqlx::PgTransaction<'_>> {
        self.pool.begin().await
    }
}
