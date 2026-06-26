use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};

use crate::{
    asset_type::AssetTypeId,
    bundle::BundleId,
    define_id,
    location::{Location, LocationId},
    permission::define_permissions,
    user::UserId,
};

#[schema]
struct Asset {
    #[schema(generated, immutable)]
    id: AssetId,
    #[schema(immutable)]
    r#type: AssetTypeId,
    alias: Option<String>,
    notes: Option<String>,
    tag: String,

    bundle: Option<BundleId>,

    home_location: LocationId,
    location: LocationId,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: UserId,
}

impl Asset {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        asset: CreateAsset,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO assets(type, alias, notes, tag, bundle, home_location, location, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id AS "id: _",
                type AS "type: _",
                alias,
                notes,
                tag,
                bundle AS "bundle: _",
                home_location AS "home_location: _",
                location AS "location: _",
                created_at,
                created_by AS "created_by: _";"#,
            asset.r#type as _,
            asset.alias,
            asset.notes,
            asset.tag,
            asset.bundle as _,
            asset.home_location as _,
            asset.location as _,
            asset.created_by as _,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: AssetId,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
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
                created_by AS "created_by: _"
            FROM assets
            WHERE id = $1;"#,
            id as _,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
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
                created_by AS "created_by: _"
            FROM assets;"#,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list_of_type(
        r#type: AssetTypeId,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> sqlx::Result<Vec<Asset>> {
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
                created_by AS "created_by: _"
            FROM assets
            WHERE type = $1;"#,
            r#type as _,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            r#"DELETE FROM assets
            WHERE id = $1;"#,
            self.id as _,
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
                notes = $3,
                tag = $4,
                bundle = $5,
                home_location = $6,
                location = $7
            WHERE id = $1
            RETURNING
                id AS "id: _",
                type AS "type: _",
                alias,
                notes,
                tag,
                bundle AS "bundle: _",
                home_location AS "home_location: _",
                location AS "location: _",
                created_at,
                created_by AS "created_by: _";"#,
            self.id as _,
            update.alias,
            update.notes,
            update.tag,
            update.bundle as _,
            update.home_location as _,
            update.location as _,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn set_location(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        location: LocationId,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"UPDATE assets
            SET location = $2
            WHERE id = $1
            RETURNING
                id AS "id: _",
                type AS "type: _",
                alias,
                notes,
                tag,
                bundle AS "bundle: _",
                home_location AS "home_location: _",
                location AS "location: _",
                created_at,
                created_by AS "created_by: _";"#,
            self.id as _,
            location as _,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_location(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Location> {
        sqlx::query_as!(
            Location,
            r#"SELECT
                locations.id AS "id: _",
                locations.name,
                locations.description,
                locations.coordinates,
                locations.created_at,
                locations.created_by AS "created_by: _"
            FROM locations
            LEFT JOIN assets
            ON locations.id = assets.location
            WHERE assets.id = $1;"#,
            self.id as _,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("assets" => View, Manage);
define_id!(AssetId);
