use uuid::Uuid;

use crate::permission::define_permissions;

pub struct Manufacturer {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

pub struct ManufacturerCreation {
    pub name: String,
    pub description: Option<String>,

    pub website: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl Manufacturer {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: ManufacturerCreation,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Manufacturer,
            "INSERT INTO manufacturers(name, description, website, email, phone)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, description, website, email, phone;",
            create.name,
            create.description,
            create.website,
            create.email,
            create.phone,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Manufacturer,
            "SELECT id, name, description, website, email, phone
            FROM manufacturers
            WHERE id = $1;",
            id
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(&self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Manufacturer,
            "DELETE FROM manufacturers
            WHERE id = $1;",
            self.id,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Manufacturer>> {
        sqlx::query_as!(
            Manufacturer,
            "SELECT id, name, description, website, email, phone
            FROM manufacturers;",
        )
        .fetch_all(&mut **txn)
        .await
    }
}

define_permissions!("manufacturers" => View, Manage);
