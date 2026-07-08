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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rust_decimal::Decimal;
    use sqlx::{PgPool, types::Text};
    use url::Url;
    use uuid::{Uuid, uuid};

    use crate::{
        PatchField,
        asset_type::{AssetType, CreateAssetType, PatchAssetType, UpdateAssetType},
    };

    const ASSET_TYPE_ID: Uuid = uuid!("f1c8508a-7c1d-436d-a867-0849dddf5f87");
    const MANUFACTURER_ID: Uuid = uuid!("3d6fd755-8d90-4a86-881f-4870049bf5f9");
    const USER_ID: Uuid = uuid!("736bcb69-ae67-4ec1-8868-cca4662aa3b1");

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_get_by_id(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");
        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        assert_eq!(asset_type.id, ASSET_TYPE_ID);
        assert_eq!(asset_type.name, "Test Asset Type");
        assert_eq!(asset_type.manufacturer, MANUFACTURER_ID);
        assert_eq!(
            asset_type.product_url,
            Some(Text(
                Url::parse("https://example.com").expect("failed to parse URL")
            ))
        );
        assert_eq!(asset_type.value, None);
        assert_eq!(asset_type.created_by, USER_ID);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_get_by_id_non_existent(pool: PgPool) {
        let non_existent_id = uuid!("76435093-ce9b-4464-8335-6e20e8a17180");

        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, non_existent_id)
            .await
            .expect("failed to get asset type");

        assert_eq!(asset_type, None);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_create(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let name: String = "Test Asset Type".into();
        let value = Some(Decimal::new(6942, 2));

        let asset_type = AssetType::create(
            &mut txn,
            CreateAssetType {
                name: name.clone(),
                manufacturer: MANUFACTURER_ID,
                product_url: None,
                value,
                created_by: USER_ID,
            },
        )
        .await
        .expect("failed to create asset type");

        assert_eq!(asset_type.name, name);
        assert_eq!(asset_type.manufacturer, MANUFACTURER_ID);
        assert_eq!(asset_type.product_url, None);
        assert_eq!(asset_type.value, value);
        assert_eq!(asset_type.created_by, USER_ID);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_create_non_existent_manufacturer(pool: PgPool) {
        let non_existent_manufacturer_id = uuid!("8abe6c36-6533-44be-ab63-1eb54d303eea");

        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::create(
            &mut txn,
            CreateAssetType {
                name: "Test Asset Type".into(),
                manufacturer: non_existent_manufacturer_id,
                product_url: None,
                value: None,
                created_by: USER_ID,
            },
        )
        .await;

        assert_matches!(asset_type, Err(_))
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_create_non_existent_user(pool: PgPool) {
        let non_existent_user_id = uuid!("76aed14a-9b1a-45b9-b125-5ae118623dfb");

        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::create(
            &mut txn,
            CreateAssetType {
                name: "Test Asset Type".into(),
                manufacturer: MANUFACTURER_ID,
                product_url: None,
                value: None,
                created_by: non_existent_user_id,
            },
        )
        .await;

        assert_matches!(asset_type, Err(_))
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_delete(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let successful = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found")
            .delete(&mut txn)
            .await
            .expect("failed to delete asset type");

        assert!(successful);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_delete_non_existent(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        // delete the asset type
        asset_type
            .clone()
            .delete(&mut txn)
            .await
            .expect("failed to delete asset type");

        // the asset type is already deleted, so deleting it again
        // should return a false success value.
        let result = asset_type
            .delete(&mut txn)
            .await
            .expect("failed to delete asset type");
        assert!(!result);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_update(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        let name: String = "New Name".into();
        let product_url = Some(Text(
            Url::parse("https://wikipedia.org").expect("failed to parse URL"),
        ));

        let new_asset_type = asset_type
            .clone()
            .update(
                &mut txn,
                UpdateAssetType {
                    name: name.clone(),
                    manufacturer: asset_type.manufacturer,
                    product_url: product_url.clone(),
                    value: asset_type.value,
                },
            )
            .await
            .expect("failed to update asset type");

        assert_eq!(new_asset_type.id, ASSET_TYPE_ID);
        assert_eq!(new_asset_type.name, name);
        assert_eq!(new_asset_type.manufacturer, asset_type.manufacturer);
        assert_eq!(new_asset_type.product_url, product_url);
        assert_eq!(new_asset_type.value, asset_type.value);
        assert_eq!(new_asset_type.created_at, asset_type.created_at);
        assert_eq!(new_asset_type.created_by, asset_type.created_by);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_update_non_existent_manufacturer(pool: PgPool) {
        let non_existent_manufacturer_id = uuid!("2ca48a56-f09f-495a-b2c2-0db2820a887d");

        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        let asset_type = asset_type
            .clone()
            .update(
                &mut txn,
                UpdateAssetType {
                    name: asset_type.name,
                    manufacturer: non_existent_manufacturer_id,
                    product_url: asset_type.product_url,
                    value: asset_type.value,
                },
            )
            .await;

        assert_matches!(asset_type, Err(_))
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_update_non_existent(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        // delete the asset type
        asset_type
            .clone()
            .delete(&mut txn)
            .await
            .expect("failed to delete asset type");

        // previously deleting the asset type means it no
        // longer exists, and an update should fail.
        let asset_type = asset_type
            .clone()
            .update(
                &mut txn,
                UpdateAssetType {
                    name: asset_type.name,
                    manufacturer: asset_type.manufacturer,
                    product_url: asset_type.product_url,
                    value: asset_type.value,
                },
            )
            .await;

        assert_matches!(asset_type, Err(_))
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_patch(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        let name: String = "New Name".into();
        let product_url = Some(Text(
            Url::parse("https://wikipedia.org").expect("failed to parse url"),
        ));

        let new_asset_type = asset_type
            .clone()
            .patch(
                &mut txn,
                PatchAssetType {
                    name: PatchField::Present(name.clone()),
                    manufacturer: PatchField::Absent,
                    product_url: PatchField::Present(product_url.clone()),
                    value: PatchField::Absent,
                },
            )
            .await
            .expect("failed to patch asset type");

        assert_eq!(new_asset_type.id, asset_type.id);
        assert_eq!(new_asset_type.name, name);
        assert_eq!(new_asset_type.manufacturer, asset_type.manufacturer);
        assert_eq!(new_asset_type.product_url, product_url);
        assert_eq!(new_asset_type.value, asset_type.value);
        assert_eq!(new_asset_type.created_at, asset_type.created_at);
        assert_eq!(new_asset_type.created_by, asset_type.created_by);
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_patch_non_existent_manufacturer(pool: PgPool) {
        let non_existent_manufacturer_id = uuid!("6e261a2c-08d4-4eb7-b1a7-dad7ade2b457");

        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found")
            .patch(
                &mut txn,
                PatchAssetType {
                    name: PatchField::Absent,
                    manufacturer: PatchField::Present(non_existent_manufacturer_id),
                    product_url: PatchField::Absent,
                    value: PatchField::Absent,
                },
            )
            .await;

        assert_matches!(asset_type, Err(_))
    }

    #[sqlx::test(fixtures(path = "fixtures", scripts("asset_types")))]
    async fn test_patch_non_existent(pool: PgPool) {
        let mut txn = pool.begin().await.expect("failed to begin transaction");

        let asset_type = AssetType::get_by_id(&mut txn, ASSET_TYPE_ID)
            .await
            .expect("failed to get asset type")
            .expect("asset type not found");

        // delete the asset type
        asset_type
            .clone()
            .delete(&mut txn)
            .await
            .expect("failed to delete asset type");

        // previously deleting the asset type means it no
        // longer exists, and a patch should fail.
        let asset_type = asset_type
            .clone()
            .patch(
                &mut txn,
                PatchAssetType {
                    name: PatchField::Absent,
                    manufacturer: PatchField::Absent,
                    product_url: PatchField::Absent,
                    value: PatchField::Absent,
                },
            )
            .await;

        assert_matches!(asset_type, Err(_))
    }
}
