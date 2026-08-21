use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgPoint;

use crate::{define_id, permission::define_permissions, user::UserId};

#[schema]
struct Location {
    #[schema(generated, immutable)]
    id: LocationId,
    name: String,
    description: Option<String>,

    coordinates: Option<PgPoint>,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: UserId,
}

impl Location {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: CreateLocation,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO locations(name, description, coordinates, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id AS "id: _", name, description, coordinates, created_at, created_by AS "created_by: _";"#,
            create.name,
            create.description,
            create.coordinates,
            create.created_by as _,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_id(
        txn: &mut sqlx::PgTransaction<'_>,
        id: LocationId,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id AS "id: _", name, description, coordinates, created_at, created_by AS "created_by: _"
            FROM locations
            WHERE id = $1;"#,
            id as _,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            "DELETE FROM locations
            WHERE id = $1;",
            self.id as _,
        )
        .execute(&mut **txn)
        .await?;

        Ok(result.rows_affected() != 0)
    }

    pub async fn list(txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id AS "id: _", name, description, coordinates, created_at, created_by AS "created_by: _"
            FROM locations;"#
        )
        .fetch_all(&mut **txn)
        .await
    }

    pub async fn update(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        update: UpdateLocation,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"UPDATE locations
            SET name = $2, description = $3, coordinates = $4
            WHERE id = $1
            RETURNING id AS "id: _", name, description, coordinates, created_at, created_by AS "created_by: _";"#,
            self.id as _,
            update.name,
            update.description,
            update.coordinates,
        )
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn patch(
        self,
        txn: &mut sqlx::PgTransaction<'_>,
        patch: PatchLocation,
    ) -> sqlx::Result<Self> {
        let (name_provided, name) = patch.name.into_case_pair();
        let (description_provided, description) = patch.description.into_nullable_case_pair();
        let (coordinates_provided, coordinates) = patch.coordinates.into_nullable_case_pair();

        sqlx::query_as!(
            Self,
            r#"UPDATE locations
            SET
                name = CASE WHEN $2 THEN $3 ELSE name END,
                description = CASE WHEN $4 THEN $5 ELSE description END,
                coordinates = CASE WHEN $6 THEN $7 ELSE coordinates END
            WHERE id = $1
            RETURNING id AS "id: _", name, description, coordinates, created_at, created_by AS "created_by: _";"#,
            self.id as _,
            name_provided,
            name,
            description_provided,
            description,
            coordinates_provided,
            coordinates,
        )
        .fetch_one(&mut **txn)
        .await
    }
}

define_permissions!("locations" => View, Manage);
define_id!(LocationId);
