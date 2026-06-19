use sqlx::PgPool;

pub mod manufacturer;
pub mod permission;
pub mod user;
pub mod user_group;

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
