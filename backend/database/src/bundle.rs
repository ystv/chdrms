use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::asset::Asset;

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

    pub async fn assets(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"SELECT id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by
            FROM assets
            WHERE bundle = $1;"#,
            &self.id,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Bundle>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, created_at, created_by
            FROM asset_bundles;"#
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn get_by_member_id(
        txn: &mut sqlx::PgTransaction<'_>,
        member: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT asset_bundles.id, asset_bundles.created_at, asset_bundles.created_by
            FROM asset_bundles
            LEFT JOIN assets
            ON asset_bundles.id = assets.bundle
            WHERE assets.id = $1;"#,
            member,
        )
        .fetch_optional(&mut **txn)
        .await
    }
}
