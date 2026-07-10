use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::permission::define_permissions;

#[schema]
struct Asset {
    #[schema(generated, immutable)]
    id: Uuid,
    #[schema(immutable)] // todo: should this be immutable?
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

pub struct AssetSummary {
    pub id: Uuid,
    pub alias: Option<String>,
    pub tag: String,
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

    pub async fn get_bundle_partner_summaries(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> Option<sqlx::Result<Vec<AssetSummary>>> {
        Some(
            sqlx::query_as!(
                AssetSummary,
                r#"SELECT id, alias, tag
                FROM assets
                WHERE bundle = $1 AND id != $2;"#,
                self.bundle?,
                self.id,
            )
            .fetch_all(&mut **txn)
            .await,
        )
    }
}

define_permissions!("assets" => View, Manage);
