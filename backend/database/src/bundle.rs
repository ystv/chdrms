use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};

use crate::{
    asset::{Asset, AssetId},
    define_id,
    user::UserId,
};

#[schema]
struct Bundle {
    #[schema(generated, immutable)]
    id: BundleId,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: UserId,
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
            RETURNING id AS "id: _", created_at, created_by AS "created_by: _";"#,
            bundle.created_by as _,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: BundleId,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id AS "id: _", created_at, created_by AS "created_by: _"
            FROM asset_bundles
            WHERE id = $1;"#,
            id as _,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            r#"DELETE FROM asset_bundles
            WHERE id = $1;"#,
            self.id as _,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn assets(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"SELECT
                id AS "id: _",
                type AS "type: _",
                alias,
                notes,
                tag,
                bundle AS "bundle: _",
                home_location AS "home_location: _",
                location AS "location: _",
                created_at,
                created_by as "created_by: _"
            FROM assets
            WHERE bundle = $1;"#,
            self.id as _,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Bundle>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id AS "id: _", created_at, created_by AS "created_by: _"
            FROM asset_bundles;"#
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn get_by_member_id(
        txn: &mut sqlx::PgTransaction<'_>,
        member: AssetId,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT asset_bundles.id AS "id: _", asset_bundles.created_at, asset_bundles.created_by AS "created_by: _"
            FROM asset_bundles
            LEFT JOIN assets
            ON asset_bundles.id = assets.bundle
            WHERE assets.id = $1;"#,
            member as _,
        )
        .fetch_optional(&mut **txn)
        .await
    }
}

define_id!(BundleId);
