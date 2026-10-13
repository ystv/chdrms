use chdrms_database_macros::schema;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use crate::{Postgres, permission::define_permissions};

#[schema]
struct Asset {
    #[schema(generated, immutable)]
    id: Uuid,
    #[schema(immutable)]
    r#type: Uuid,
    alias: Option<String>,
    notes: Option<String>,
    tag: String,

    bundle: Option<Uuid>,

    home_location: Uuid,
    location: Uuid,

    #[schema(generated, immutable)]
    created_at: DateTime<Utc>,
    #[schema(immutable)]
    created_by: Uuid,
}

pub trait AssetRepository {
    fn get_assets_by_bundle_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Vec<Asset>, GetAssetsByBundleIdError>> + Send;

    fn create_asset(
        &self,
        asset: &CreateAsset,
    ) -> impl Future<Output = Result<Asset, CreateAssetError>> + Send;

    fn get_asset_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Asset>, GetAssetByIdError>> + Send;

    fn delete_asset_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), DeleteAssetByIdError>> + Send;

    fn update_asset_by_id(
        &self,
        id: Uuid,
        asset: &UpdateAsset,
    ) -> impl Future<Output = Result<Asset, UpdateAssetByIdError>> + Send;

    fn list_assets(&self) -> impl Future<Output = Result<Vec<Asset>, ListAssetsError>> + Send;

    fn get_assets_by_type_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Vec<Asset>, GetAssetsByTypeIdError>> + Send;

    fn set_asset_location_by_id(
        &self,
        asset: Uuid,
        location: Uuid,
    ) -> impl Future<Output = Result<Asset, SetAssetLocationByIdError>> + Send;
}

#[derive(Debug, Error)]
pub enum GetAssetsByBundleIdError {
    #[error("backend error")]
    Backend,
}

#[derive(Debug, Error)]
pub enum CreateAssetError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
}

#[derive(Debug, Error)]
pub enum GetAssetByIdError {
    #[error("backend error")]
    Backend,
}

#[derive(Debug, Error)]
pub enum DeleteAssetByIdError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum UpdateAssetByIdError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum ListAssetsError {
    #[error("backend error")]
    Backend,
}

#[derive(Debug, Error)]
pub enum GetAssetsByTypeIdError {
    #[error("backend error")]
    Backend,
}

#[derive(Debug, Error)]
pub enum SetAssetLocationByIdError {
    #[error("backend error")]
    Backend,
    #[error("relationship error")]
    Relationship,
    #[error("not found")]
    NotFound,
}

impl AssetRepository for Postgres {
    async fn get_assets_by_bundle_id(
        &self,
        id: Uuid,
    ) -> Result<Vec<Asset>, GetAssetsByBundleIdError> {
        sqlx::query_as!(
            Asset,
            r#"SELECT id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by
            FROM assets
            WHERE bundle = $1;"#,
            id,
        )
        .fetch_all(&mut *self.transaction().await?)
        .await
        .map_err(Into::into)
    }

    async fn create_asset(&self, asset: &CreateAsset) -> Result<Asset, CreateAssetError> {
        let mut txn = self.transaction().await?;

        let asset = sqlx::query_as!(
            Asset,
            r#"INSERT INTO assets(type, alias, notes, tag, bundle, home_location, location, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by;"#,
            asset.r#type,
            asset.alias,
            asset.notes,
            asset.tag,
            asset.bundle,
            asset.home_location,
            asset.location,
            asset.created_by,
        )
        .fetch_one(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(asset)
    }

    async fn get_asset_by_id(&self, id: Uuid) -> Result<Option<Asset>, GetAssetByIdError> {
        sqlx::query_as!(
            Asset,
            r#"SELECT id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by
            FROM assets
            WHERE id = $1;"#,
            id,
        )
         .fetch_optional(&mut *self.transaction().await?)
         .await
         .map_err(Into::into)
    }

    async fn delete_asset_by_id(&self, id: Uuid) -> Result<(), DeleteAssetByIdError> {
        let mut txn = self.transaction().await?;

        let result = sqlx::query_as!(
            Asset,
            r#"DELETE FROM assets
            WHERE id = $1;"#,
            id,
        )
        .execute(&mut *txn)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DeleteAssetByIdError::NotFound);
        }

        txn.commit().await?;

        Ok(())
    }

    async fn update_asset_by_id(
        &self,
        id: Uuid,
        asset: &UpdateAsset,
    ) -> Result<Asset, UpdateAssetByIdError> {
        let mut txn = self.transaction().await?;

        let asset = sqlx::query_as!(
            Asset,
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
                id,
                type,
                alias,
                notes,
                tag,
                bundle,
                home_location,
                location,
                created_at,
                created_by;"#,
            id,
            asset.alias,
            asset.notes,
            asset.tag,
            asset.bundle,
            asset.home_location,
            asset.location,
        )
        // todo: switch this to fetch_one as it throws
        //       an error when not found, which is more
        //       idiomatic than handling an Option.
        .fetch_optional(&mut *txn)
        .await?;

        let Some(asset) = asset else {
            return Err(UpdateAssetByIdError::NotFound);
        };

        txn.commit().await?;

        Ok(asset)
    }

    async fn list_assets(&self) -> Result<Vec<Asset>, ListAssetsError> {
        sqlx::query_as!(
            Asset,
            r#"SELECT id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by
            FROM assets;"#,
        )
        .fetch_all(&mut *self.transaction().await?)
        .await
        .map_err(Into::into)
    }

    async fn get_assets_by_type_id(&self, id: Uuid) -> Result<Vec<Asset>, GetAssetsByTypeIdError> {
        sqlx::query_as!(
            Asset,
            r#"SELECT
                id,
                type,
                alias,
                notes,
                tag,
                bundle,
                home_location,
                location,
                created_at,
                created_by
            FROM assets
            WHERE type = $1;"#,
            id,
        )
        .fetch_all(&mut *self.transaction().await?)
        .await
        .map_err(Into::into)
    }

    async fn set_asset_location_by_id(
        &self,
        asset: Uuid,
        location: Uuid,
    ) -> Result<Asset, SetAssetLocationByIdError> {
        let mut txn = self.transaction().await?;

        let asset = sqlx::query_as!(
            Asset,
            r#"UPDATE assets
            SET location = $2
            WHERE id = $1
            RETURNING id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by;"#,
            asset,
            location,
        )
         .fetch_one(&mut *txn)
         .await?;

        txn.commit().await?;

        Ok(asset)
    }
}

pub enum AssetIdentifier {
    Id(Uuid),
    Tag(String),
}

impl From<sqlx::Error> for GetAssetsByBundleIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        GetAssetsByBundleIdError::Backend
    }
}

impl From<sqlx::Error> for CreateAssetError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => CreateAssetError::Relationship,
                _ => CreateAssetError::Backend,
            },
            _ => CreateAssetError::Backend,
        }
    }
}

impl From<sqlx::Error> for GetAssetByIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        GetAssetByIdError::Backend
    }
}

impl From<sqlx::Error> for DeleteAssetByIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => DeleteAssetByIdError::Relationship,
                _ => DeleteAssetByIdError::Backend,
            },
            _ => DeleteAssetByIdError::Backend,
        }
    }
}

impl From<sqlx::Error> for UpdateAssetByIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => UpdateAssetByIdError::Relationship,
                _ => UpdateAssetByIdError::Backend,
            },
            _ => UpdateAssetByIdError::Backend,
        }
    }
}

impl From<sqlx::Error> for ListAssetsError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        ListAssetsError::Backend
    }
}

impl From<sqlx::Error> for GetAssetsByTypeIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        GetAssetsByTypeIdError::Backend
    }
}

impl From<sqlx::Error> for SetAssetLocationByIdError {
    fn from(err: sqlx::Error) -> Self {
        error!(error = ?err, "database error occurred");
        match err {
            sqlx::Error::Database(err) => match err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => {
                    SetAssetLocationByIdError::Relationship
                }
                _ => SetAssetLocationByIdError::Backend,
            },
            _ => SetAssetLocationByIdError::Backend,
        }
    }
}

impl Asset {
    pub async fn asset_exists(
        txn: &mut sqlx::PgTransaction<'_>,
        identifier: &AssetIdentifier,
    ) -> sqlx::Result<bool> {
        match identifier {
            AssetIdentifier::Id(id) => sqlx::query_scalar!(
                r#"SELECT EXISTS (
                    SELECT 1
                    FROM assets
                    WHERE id = $1
                ) AS "exists!: bool""#,
                id,
            ),
            AssetIdentifier::Tag(tag) => sqlx::query_scalar!(
                r#"SELECT EXISTS (
                    SELECT 1
                    FROM assets
                    WHERE tag = $1
                ) AS "exists!: bool""#,
                tag,
            ),
        }
        .fetch_one(&mut **txn)
        .await
    }

    pub async fn get_by_tag(
        txn: &mut sqlx::PgTransaction<'_>,
        tag: String,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, type, alias, notes, tag, bundle, home_location, location, created_at, created_by
            FROM assets
            WHERE tag = $1;"#,
            tag,
        )
        .fetch_optional(&mut **txn)
        .await
    }
}

define_permissions!("assets" => View, Manage);
