use s3::Credentials;
use serde::Serialize;
use url::Url;
use utoipa::ToSchema;

use crate::config::{S3Config, StorageConfig};

#[derive(Clone)]
pub struct Storage {
    backend: StorageBackend,
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
        }
    }

    pub fn get_test_document(&self) -> PresignedRequest {
        self.backend.get_download_url("test.txt")
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

    fn get_download_url(&self, key: impl Into<String>) -> PresignedRequest {
        match self {
            StorageBackend::S3 { bucket, client } => client
                .objects()
                .presign_get(bucket, key)
                .build()
                .unwrap()
                .into(),
        }
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
