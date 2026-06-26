use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{asset_type::AssetType, permission::define_permissions};

#[schema]
struct Manufacturer {
    #[schema(generated, immutable)]
    id: Uuid,
    name: String,
    description: Option<String>,

    website: Option<String>,
    email: Option<String>,
    phone: Option<String>,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

impl Manufacturer {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: CreateManufacturer,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "INSERT INTO manufacturers(name, description, website, email, phone, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, description, website, email, phone, created_at, created_by;",
            create.name,
            create.description,
            create.website,
            create.email,
            create.phone,
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
            "SELECT id, name, description, website, email, phone, created_at, created_by
            FROM manufacturers
            WHERE id = $1;",
            id
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            "DELETE FROM manufacturers
            WHERE id = $1;",
            self.id,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, name, description, website, email, phone, created_at, created_by
            FROM manufacturers;",
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn list_asset_types(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
    ) -> sqlx::Result<Vec<AssetType>> {
        sqlx::query_as!(
            AssetType,
            r#"SELECT id, name, manufacturer, product_url AS "product_url: _", value, created_at, created_by
            FROM asset_types
            WHERE manufacturer = $1;"#,
            self.id
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn update(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        update: UpdateManufacturer,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "UPDATE manufacturers
            SET name = $2, description = $3, website = $4, email = $5, phone = $6
            WHERE id = $1
            RETURNING id, name, description, website, email, phone, created_at, created_by;",
            self.id,
            update.name,
            update.description,
            update.website,
            update.email,
            update.phone,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn patch(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        patch: PatchManufacturer,
    ) -> sqlx::Result<Self> {
        let (name_provided, name) = patch.name.into_case_pair();
        let (description_provided, description) = patch.description.into_nullable_case_pair();
        let (website_provided, website) = patch.website.into_nullable_case_pair();
        let (email_provided, email) = patch.email.into_nullable_case_pair();
        let (phone_provided, phone) = patch.phone.into_nullable_case_pair();

        sqlx::query_as!(
            Self,
            "UPDATE manufacturers
            SET
                name = CASE WHEN $1 THEN $2 ELSE name END,
                description = CASE WHEN $3 THEN $4 ELSE description END,
                website = CASE WHEN $5 THEN $6 ELSE website END,
                email = CASE WHEN $7 THEN $8 ELSE email END,
                phone = CASE WHEN $9 THEN $10 ELSE phone END
            WHERE id = $11
            RETURNING id, name, description, website, email, phone, created_at, created_by;",
            name_provided,
            name,
            description_provided,
            description,
            website_provided,
            website,
            email_provided,
            email,
            phone_provided,
            phone,
            self.id,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("manufacturers" => View, Manage);
