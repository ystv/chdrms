use std::collections::HashMap;

use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{Colour, permission::define_permissions};

#[schema]
struct Label {
    #[schema(generated, immutable)]
    id: Uuid,
    name: String,
    description: Option<String>,
    colour: Option<Colour>,

    blocking: bool,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct LabelAndBlocking {
    pub id: Uuid,
    pub blocking: bool,
}

impl Label {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        label: CreateLabel,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO labels(name, description, colour, blocking, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, colour AS "colour: _", blocking, created_at, created_by;"#,
            label.name,
            label.description,
            label.colour as _,
            label.blocking,
            label.created_by,
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
            r#"SELECT id, name, description, colour AS "colour: _", blocking, created_at, created_by
            FROM labels
            WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn list_asset_labels(
        txn: &mut sqlx::PgTransaction<'_>,
        asset: Uuid,
    ) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT labels.id, labels.name, labels.description, labels.colour AS "colour: _", labels.blocking, labels.created_at, labels.created_by
            FROM labels
            LEFT JOIN asset_labels
            ON labels.id = asset_labels.label AND asset_labels.asset = $1;"#,
            asset,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list_label_and_blocking_for_assets(
        txn: &mut sqlx::PgTransaction<'_>,
        assets: Vec<Uuid>,
    ) -> sqlx::Result<HashMap<Uuid, Vec<LabelAndBlocking>>> {
        let records = sqlx::query!(
            r#"SELECT labels.id AS "label", asset_labels.asset AS "asset", labels.blocking AS "blocking"
            FROM labels
            LEFT JOIN asset_labels
            ON labels.id = asset_labels.label
            WHERE asset_labels.asset = ANY($1);"#,
            &assets,
        )
        .fetch_all(&mut **txn)
        .await?;

        let mut labels_by_asset: HashMap<Uuid, Vec<LabelAndBlocking>> = HashMap::new();

        for row in records {
            labels_by_asset
                .entry(row.asset)
                .or_default()
                .push(LabelAndBlocking {
                    id: row.label,
                    blocking: row.blocking,
                });
        }

        Ok(labels_by_asset)
    }

    pub async fn list_label_and_blocking_for_asset(
        txn: &mut sqlx::PgTransaction<'_>,
        asset: Uuid,
    ) -> sqlx::Result<Vec<LabelAndBlocking>> {
        sqlx::query_as!(
            LabelAndBlocking,
            r#"SELECT labels.id, labels.blocking
            FROM labels
            LEFT JOIN asset_labels
            ON labels.id = asset_labels.label AND asset_labels.asset = $1;"#,
            asset,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list_blocking_label_ids_for_assets(
        txn: &mut sqlx::PgTransaction<'_>,
        assets: Vec<Uuid>,
    ) -> sqlx::Result<HashMap<Uuid, Vec<Uuid>>> {
        let records = sqlx::query!(
            r#"SELECT labels.id AS "label", asset_labels.asset AS "asset"
            FROM labels
            LEFT JOIN asset_labels
            ON labels.id = asset_labels.label
            WHERE asset_labels.asset = ANY($1) AND labels.blocking;"#,
            &assets,
        )
        .fetch_all(&mut **txn)
        .await?;

        let mut labels_by_asset: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for row in records {
            labels_by_asset
                .entry(row.asset)
                .or_default()
                .push(row.label);
        }

        Ok(labels_by_asset)
    }

    pub async fn list_blocking_label_ids_for_asset(
        txn: &mut sqlx::PgTransaction<'_>,
        asset: Uuid,
    ) -> sqlx::Result<Vec<Uuid>> {
        sqlx::query_scalar!(
            r#"SELECT labels.id
            FROM labels
            LEFT JOIN asset_labels
            ON labels.id = asset_labels.label AND asset_labels.asset = $1
            WHERE labels.blocking;"#,
            asset,
        )
        .fetch_all(&mut **txn)
        .await
    }
}

define_permissions!("labels" => View, Manage);
