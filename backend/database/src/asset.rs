use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{location::Location, permission::define_permissions};

#[schema]
struct Asset {
    #[schema(generated, immutable)]
    id: Uuid,
    #[schema(immutable)]
    r#type: Uuid,
    alias: Option<String>,
    tag: String,

    bundle: Option<Uuid>,

    home_location: Uuid,
    location: Uuid,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

impl Asset {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        asset: CreateAsset,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO assets(type, alias, tag, bundle, home_location, location, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, type, alias, tag, bundle, home_location, location, created_at, created_by;"#,
            asset.r#type,
            asset.alias,
            asset.tag,
            asset.bundle,
            asset.home_location,
            asset.location,
            asset.created_by,
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
            r#"SELECT id, type, alias, tag, bundle, home_location, location, created_at, created_by
            FROM assets
            WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, type, alias, tag, bundle, home_location, location, created_at, created_by
            FROM assets;"#,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            r#"DELETE FROM assets
            WHERE id = $1;"#,
            self.id,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn update(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        update: UpdateAsset,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"UPDATE assets
            SET
                alias = $2,
                tag = $3,
                bundle = $4,
                home_location = $5,
                location = $6
            WHERE id = $1
            RETURNING
                id,
                type,
                alias,
                tag,
                bundle,
                home_location,
                location,
                created_at,
                created_by;"#,
            self.id,
            update.alias,
            update.tag,
            update.bundle,
            update.home_location,
            update.location,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn set_location(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        location: Uuid,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"UPDATE assets
            SET location = $2
            WHERE id = $1
            RETURNING id, type, alias, tag, bundle, home_location, location, created_at, created_by;"#,
            &self.id,
            location,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_location(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Location> {
        sqlx::query_as!(
            Location,
            r#"SELECT locations.id, locations.name, locations.description, locations.coordinates, locations.created_at, locations.created_by
            FROM locations
            LEFT JOIN assets
            ON locations.id = assets.location
            WHERE assets.id = $1;"#,
            &self.id,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("assets" => View, Manage);
