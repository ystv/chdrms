use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use crate::Postgres;

#[schema]
struct Bundle {
    #[schema(generated, immutable)]
    id: Uuid,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

pub trait BundleRepository {
    fn get_bundle_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Bundle>, GetBundleByIdError>> + Send;

    fn create_bundle(
        &self,
        bundle: &CreateBundle,
    ) -> impl Future<Output = Result<Bundle, CreateBundleError>> + Send;

    fn delete_bundle_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), DeleteBundleError>> + Send;

    fn list_bundles(&self) -> impl Future<Output = Result<Vec<Bundle>, ListBundlesError>> + Send;
}

#[derive(Debug, Error)]
pub enum GetBundleByIdError {
    #[error("backend error")]
    Backend,
}

#[derive(Debug, Error)]
pub enum CreateBundleError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
}

#[derive(Debug, Error)]
pub enum DeleteBundleError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
    #[error("bundle not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum ListBundlesError {
    #[error("backend error")]
    Backend,
}

impl BundleRepository for Postgres {
    async fn get_bundle_by_id(&self, id: Uuid) -> Result<Option<Bundle>, GetBundleByIdError> {
        sqlx::query_as!(
            Bundle,
            r#"SELECT id, created_at, created_by
            FROM asset_bundles
            WHERE id = $1;"#,
            id
        )
        .fetch_optional(&mut *self.transaction().await?)
        .await
        .map_err(Into::into)
    }

    async fn create_bundle(&self, bundle: &CreateBundle) -> Result<Bundle, CreateBundleError> {
        let mut txn = self.transaction().await?;

        let bundle = sqlx::query_as!(
            Bundle,
            r#"INSERT INTO asset_bundles(created_by)
            VALUES ($1)
            RETURNING id, created_at, created_by;"#,
            bundle.created_by,
        )
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(bundle)
    }

    async fn delete_bundle_by_id(&self, id: Uuid) -> Result<(), DeleteBundleError> {
        let mut txn = self.transaction().await?;

        let result = sqlx::query_as!(
            Self,
            r#"DELETE FROM asset_bundles
            WHERE id = $1;"#,
            id,
        )
        .execute(&mut *txn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DeleteBundleError::NotFound);
        }

        txn.commit().await?;

        Ok(())
    }

    async fn list_bundles(&self) -> Result<Vec<Bundle>, ListBundlesError> {
        sqlx::query_as!(
            Bundle,
            r#"SELECT id, created_at, created_by
            FROM asset_bundles;"#
        )
        .fetch_all(&mut *self.transaction().await?)
        .await
        .map_err(Into::into)
    }
}

impl From<sqlx::Error> for GetBundleByIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        GetBundleByIdError::Backend
    }
}

impl From<sqlx::Error> for CreateBundleError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => CreateBundleError::Relationship,
                _ => CreateBundleError::Backend,
            },
            _ => CreateBundleError::Backend,
        }
    }
}

impl From<sqlx::Error> for DeleteBundleError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => DeleteBundleError::Relationship,
                _ => DeleteBundleError::Backend,
            },
            _ => DeleteBundleError::Backend,
        }
    }
}

impl From<sqlx::Error> for ListBundlesError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        ListBundlesError::Backend
    }
}
