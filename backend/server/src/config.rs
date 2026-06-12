use std::{collections::HashMap, path::Path};

use axum_extra::extract::cookie::Key;
use openidconnect::{ClientId, ClientSecret, IssuerUrl, url::Url};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub base_url: Url,
    #[serde(rename = "oidc_provider")]
    pub oidc_providers: HashMap<String, OIDCProviderConfig>,
}

#[derive(Clone, Deserialize)]
pub struct OIDCProviderConfig {
    pub name: String,
    pub issuer_url: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub allow_registration: bool,
    pub auto_account_linking: bool,
}

pub async fn load_configuration(path: impl AsRef<Path>) -> AppConfig {
    tracing::debug!(path = ?path.as_ref(), "loading configuration");
    // TODO: do we want/need better error handling here?
    let s = tokio::fs::read_to_string(path)
        .await
        .expect("failed to read config file");
    toml::from_str(&s).expect("failed to parse config")
}

// TODO: support loading from environment variable?
pub async fn load_key(path: impl AsRef<Path>) -> Key {
    if path.as_ref().exists() {
        let data = tokio::fs::read(path)
            .await
            .expect("failed to read secret key file");
        Key::from(&data)
    } else {
        let key = Key::generate();
        tokio::fs::write(path, key.master())
            .await
            .expect("failed to write secret key file");
        key
    }
}
