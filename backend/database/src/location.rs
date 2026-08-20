use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgPoint;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use crate::{Postgres, permission::define_permissions};

#[schema]
struct Location {
    #[schema(generated, immutable)]
    id: Uuid,
    name: String,
    description: Option<String>,

    coordinates: Option<PgPoint>,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

pub trait LocationRepository {
    fn get_location_by_asset_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Location, GetLocationByAssetIdError>> + Send;

    fn create_location(
        &self,
        location: &CreateLocation,
    ) -> impl Future<Output = Result<Location, CreateLocationError>> + Send;
}

#[derive(Debug, Error)]
pub enum GetLocationByAssetIdError {
    #[error("backend error")]
    Backend,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum CreateLocationError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
}

impl LocationRepository for Postgres {
    async fn get_location_by_asset_id(
        &self,
        id: Uuid,
    ) -> Result<Location, GetLocationByAssetIdError> {
        sqlx::query_as!(
            Location,
            r#"SELECT locations.id, locations.name, locations.description, locations.coordinates, locations.created_at, locations.created_by
            FROM locations
            LEFT JOIN assets
            ON locations.id = assets.location
            WHERE assets.id = $1;"#,
            id,
        )
        .fetch_one(&mut *self.transaction().await?)
        .await.map_err(Into::into)
    }

    async fn create_location(
        &self,
        location: &CreateLocation,
    ) -> Result<Location, CreateLocationError> {
        let mut txn = self.transaction().await?;

        let location = sqlx::query_as!(
            Location,
            "INSERT INTO locations(name, description, coordinates, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, coordinates, created_at, created_by;",
            location.name,
            location.description,
            location.coordinates,
            location.created_by,
        )
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(location)
    }
}

impl From<sqlx::Error> for GetLocationByAssetIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::RowNotFound => GetLocationByAssetIdError::NotFound,
            _ => GetLocationByAssetIdError::Backend,
        }
    }
}

impl From<sqlx::Error> for CreateLocationError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => CreateLocationError::Relationship,
                _ => CreateLocationError::Backend,
            },
            _ => CreateLocationError::Backend,
        }
    }
}

impl Location {
    pub async fn create(
        txn: &mut sqlx::PgTransaction<'_>,
        create: CreateLocation,
    ) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            "INSERT INTO locations(name, description, coordinates, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, coordinates, created_at, created_by;",
            create.name,
            create.description,
            create.coordinates,
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
            "SELECT id, name, description, coordinates, created_at, created_by
            FROM locations
            WHERE id = $1;",
            id,
        )
        .fetch_optional(&mut **txn)
        .await
    }

    pub async fn delete(self, txn: &mut sqlx::PgTransaction<'_>) -> sqlx::Result<bool> {
        let result = sqlx::query_as!(
            Self,
            "DELETE FROM locations
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
            "SELECT id, name, description, coordinates, created_at, created_by
            FROM locations;"
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
            "UPDATE locations
            SET name = $2, description = $3, coordinates = $4
            WHERE id = $1
            RETURNING id, name, description, coordinates, created_at, created_by;",
            self.id,
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
            "UPDATE locations
            SET
                name = CASE WHEN $2 THEN $3 ELSE name END,
                description = CASE WHEN $4 THEN $5 ELSE description END,
                coordinates = CASE WHEN $6 THEN $7 ELSE coordinates END
            WHERE id = $1
            RETURNING id, name, description, coordinates, created_at, created_by;",
            self.id,
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
