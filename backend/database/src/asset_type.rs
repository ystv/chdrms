use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::types::Text;
use url::Url;
use uuid::Uuid;

use crate::permission::define_permissions;

#[schema]
struct AssetType {
    #[schema(generated, immutable)]
    id: Uuid,
    name: String,
    manufacturer: Uuid,

    product_url: Option<Text<Url>>,
    value: Option<Decimal>,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

impl AssetType {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: CreateAssetType,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO asset_types(name, manufacturer, product_url, value, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by;"#,
            create.name,
            create.manufacturer,
            create.product_url as _,
            create.value,
            create.created_by,
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
            r#"SELECT id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by
            FROM asset_types
            WHERE id = $1;"#,
            id
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            "DELETE FROM asset_types
            WHERE id = $1;",
            self.id
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by
            FROM asset_types;"#,
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn update(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        update: UpdateAssetType,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"UPDATE asset_types
            SET name = $2, manufacturer = $3, product_url = $4, value = $5
            WHERE id = $1
            RETURNING id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by;"#,
            self.id,
            update.name,
            update.manufacturer,
            update.product_url as _,
            update.value,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn patch(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        patch: PatchAssetType,
    ) -> sqlx::Result<Self> {
        let (name_provided, name) = patch.name.into_case_pair();
        let (manufacturer_provided, manufacturer) = patch.manufacturer.into_case_pair();
        let (product_url_provided, product_url) = patch.product_url.into_nullable_case_pair();
        let (value_provided, value) = patch.value.into_nullable_case_pair();

        sqlx::query_as!(
            Self,
            r#"UPDATE asset_types
            SET
                name = CASE WHEN $1 THEN $2 ELSE name END,
                manufacturer = CASE WHEN $3 THEN $4 ELSE manufacturer END,
                product_url = CASE WHEN $5 THEN $6 ELSE product_url END,
                value = CASE WHEN $7 THEN $8 ELSE value END
            WHERE id = $9
            RETURNING id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by;"#,
            name_provided,
            name,
            manufacturer_provided,
            manufacturer,
            product_url_provided,
            product_url as _,
            value_provided,
            value,
            self.id,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("asset_types" => View, Manage);
