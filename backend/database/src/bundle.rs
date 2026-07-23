use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[schema]
struct Bundle {
    #[schema(generated, immutable)]
    id: Uuid,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

impl Bundle {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        bundle: CreateBundle,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO asset_bundles(created_by)
            VALUES ($1)
            RETURNING id, created_at, created_by;"#,
            bundle.created_by,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, created_at, created_by
            FROM asset_bundles
            WHERE id = $1;"#,
            id
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            r#"DELETE FROM asset_bundles
            WHERE id = $1;"#,
            self.id,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }
}
