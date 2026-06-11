use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct OIDCProviderConfig {
    pub name: String,
    pub issuer_url: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub allow_registration: bool,
    pub auto_account_linking: bool,
}
