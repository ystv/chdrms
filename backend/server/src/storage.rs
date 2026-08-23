use std::time::{Duration, Instant};

use bytes::Bytes;
use moka::{Expiry, future::Cache};
use s3::Credentials;
use serde::Serialize;
use tokio_stream::Stream;
use url::Url;
use utoipa::ToSchema;

use crate::config::{S3Config, StorageConfig};

#[derive(Clone)]
pub struct Storage {
    backend: StorageBackend,
    cache: Cache<String, (Duration, PresignedRequest)>,
}

#[derive(Clone)]
enum StorageBackend {
    S3 { bucket: String, client: s3::Client },
}

#[derive(Debug, Clone, ToSchema, Serialize)]
pub struct PresignedRequest {
    pub method: String,
    pub url: Url,
}

struct DurationExpiry;

impl<K, V> Expiry<K, (Duration, V)> for DurationExpiry {
    fn expire_after_create(
        &self,
        _: &K,
        (duration, _): &(Duration, V),
        _: Instant,
    ) -> Option<Duration> {
        if duration.is_zero() {
            return None;
        }
        Some(*duration)
    }
}

impl From<s3::types::PresignedRequest> for PresignedRequest {
    fn from(request: s3::types::PresignedRequest) -> Self {
        Self {
            method: request.method.to_string(),
            url: request.url,
        }
    }
}

impl From<Url> for PresignedRequest {
    fn from(url: Url) -> Self {
        Self {
            method: "GET".to_string(),
            url,
        }
    }
}

impl Storage {
    pub fn new(config: &StorageConfig) -> Self {
        Self {
            backend: StorageBackend::new(config),
            cache: Cache::builder().expire_after(DurationExpiry).build(),
        }
    }

    async fn fetch_download_url(
        &self,
        key: impl Into<String>,
        lifetime: Duration,
    ) -> PresignedRequest {
        let key = key.into();

        let (_, presigned_request) = self
            .cache
            .get_with_by_ref(&key, async {
                (
                    lifetime - Duration::from_mins(5),
                    self.backend.fetch_download_url(&key, lifetime).await,
                )
            })
            .await;

        presigned_request
    }
}

impl StorageBackend {
    fn new(config: &StorageConfig) -> Self {
        match config {
            StorageConfig::S3(config) => StorageBackend::S3 {
                bucket: config.bucket.clone(),
                client: make_s3_client(config).unwrap(), // todo: aw hell naw
            },
        }
    }

    async fn fetch_download_url(&self, key: &str, lifetime: Duration) -> PresignedRequest {
        match self {
            StorageBackend::S3 { bucket, client } => client
                .objects()
                .presign_get(bucket, key)
                .expires_in(lifetime)
                .unwrap()
                .build_async()
                .await
                .unwrap()
                .into(),
        }
    }

    async fn upload<S, E>(&self, key: impl Into<String>, content_type: impl Into<String>, stream: S)
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        match self {
            StorageBackend::S3 { bucket, client } => client
                .objects()
                .put(bucket, key)
                .body_stream(stream)
                .content_type(content_type)
                .unwrap()
                .send()
                .await
                .unwrap(),
        };
    }
}

fn make_s3_client(config: &S3Config) -> Result<s3::Client, Box<s3::Error>> {
    let mut client = s3::Client::builder(&config.endpoint)?.region(&config.region);

    if config.path_style {
        client = client.addressing_style(s3::AddressingStyle::Path)
    }

    if let Some(access_key) = &config.access_key_id
        && let Some(secret_key) = &config.secret_access_key
    {
        client = client.auth(s3::Auth::Static(Credentials::new(access_key, secret_key)?))
    }

    Ok(client.build()?)
}

impl StorageObject for chdrms_database::StorageKey {
    fn key(&self) -> String {
        self.key().to_owned()
    }
}

pub trait StorageObject {
    fn key(&self) -> String;

    fn fetch_download_url(
        &self,
        storage: &Storage,
        lifetime: Duration,
    ) -> impl Future<Output = PresignedRequest> + Send {
        let key = self.key();
        async move { storage.fetch_download_url(key, lifetime).await }
    }

    fn upload<S, E>(
        &self,
        storage: &Storage,
        content_type: impl Into<String>,
        stream: S,
    ) -> impl Future<Output = ()> + Send
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let key = self.key();
        let content_type = content_type.into();
        async { storage.backend.upload(key, content_type, stream).await }
    }
}
