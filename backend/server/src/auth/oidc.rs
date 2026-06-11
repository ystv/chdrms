use axum::response::Redirect;
use axum_extra::extract::{
    CookieJar, PrivateCookieJar,
    cookie::{Cookie, SameSite},
};
use chdrms_database::user::{User, UserCreation};
use openidconnect::{
    AuthorizationCode, ClaimsVerificationError, CsrfToken, DiscoveryError, EmptyAdditionalClaims,
    EndpointMaybeSet, EndpointNotSet, EndpointSet, HttpClientError, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RequestTokenError, StandardErrorResponse, TokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType,
        CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
        CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenResponse,
    },
    reqwest,
    url::Url,
};

use crate::{auth::SESSION_COOKIE, config::OIDCProviderConfig, error::Result};

type ConfiguredCoreClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    openidconnect::StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    openidconnect::core::CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,      // HasAuthUrl - set by from_provider_metadata
    EndpointNotSet,   // HasDeviceAuthUrl
    EndpointNotSet,   // HasIntrospectionUrl
    EndpointNotSet,   // HasRevocationUrl
    EndpointMaybeSet, // HasTokenUrl - maybe set by provider
    EndpointMaybeSet, // HasUserInfoUrl - maybe set by provider
>;

#[derive(Debug, thiserror::Error)]
pub enum AuthSetupError {
    #[error("discovery error: {0}")]
    DiscoveryError(#[from] DiscoveryError<HttpClientError<openidconnect::reqwest::Error>>),
    #[error("url parse error: {0}")]
    ParseError(#[from] openidconnect::url::ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing `{0}` cookie")]
    MissingCookie(&'static str),
    #[error("invalid state")]
    InvalidState,
    #[error("token request error: {0}")]
    TokenRequest(
        #[from]
        RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),
    #[error("oidc configuration error: {0}")]
    Configuration(#[from] openidconnect::ConfigurationError),
    #[error("oidc token claim verification error: {0}")]
    ClaimVerification(#[from] ClaimsVerificationError),
    #[error("server did not return an id token")]
    MissingIDToken,
    #[error("server did not return required claim: {0}")]
    MissingClaim(&'static str),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("auto-registration and account linking disabled")]
    UserNotFound,
}

#[derive(Clone)]
pub struct OIDCProvider {
    config: OIDCProviderConfig,
    oidc_client: ConfiguredCoreClient,
    client: reqwest::Client,
}

const OIDC_CSRF_COOKIE: &str = super::cookie_name!("oidc_csrf");
const OIDC_PKCE_COOKIE: &str = super::cookie_name!("oidc_pkce");
const OIDC_NONCE_COOKIE: &str = super::cookie_name!("oidc_nonce");

impl OIDCProvider {
    pub async fn new(
        base_url: &Url,
        config: OIDCProviderConfig,
        client: reqwest::Client,
    ) -> Result<Self, AuthSetupError> {
        let provider_metadata =
            CoreProviderMetadata::discover_async(config.issuer_url.clone(), &client).await?;
        let oidc_client = CoreClient::from_provider_metadata(
            provider_metadata,
            config.client_id.clone(),
            Some(config.client_secret.clone()),
        )
        .set_redirect_uri(RedirectUrl::from_url(
            base_url.join(&format!("/api/auth/{}/callback", config.name))?,
        ));
        Ok(Self {
            config,
            oidc_client,
            client,
        })
    }

    pub fn prepare_auth(&self, jar: PrivateCookieJar) -> (PrivateCookieJar, Redirect) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token, nonce) = self
            .oidc_client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(pkce_challenge)
            .url();

        let csrf_cookie = {
            let mut cookie = Cookie::new(OIDC_CSRF_COOKIE, csrf_token.secret().clone());
            cookie.set_http_only(true);
            cookie.set_same_site(Some(SameSite::Lax));
            cookie.set_path("/");
            cookie
        };
        let pkce_cookie = {
            let mut cookie = Cookie::new(OIDC_PKCE_COOKIE, pkce_verifier.secret().clone());
            cookie.set_http_only(true);
            cookie.set_same_site(Some(SameSite::Lax));
            cookie.set_path("/");
            cookie
        };
        let nonce_cookie = {
            let mut cookie = Cookie::new(OIDC_NONCE_COOKIE, nonce.secret().clone());
            cookie.set_http_only(true);
            cookie.set_same_site(Some(SameSite::Lax));
            cookie.set_path("/");
            cookie
        };
        let jar = jar.add(csrf_cookie).add(pkce_cookie).add(nonce_cookie);
        (jar, Redirect::to(auth_url.as_str()))
    }

    pub async fn complete_auth(
        &self,
        txn: &mut sqlx::PgTransaction<'_>,
        private_jar: PrivateCookieJar,
        jar: CookieJar,
        code: String,
        state: String,
    ) -> Result<(PrivateCookieJar, CookieJar, Redirect), AuthError> {
        let (csrf_token, pkce_verifier, nonce) = {
            let Some(mut csrf_cookie) = private_jar.get(OIDC_CSRF_COOKIE) else {
                return Err(AuthError::MissingCookie(OIDC_CSRF_COOKIE));
            };
            let Some(mut pkce_cookie) = private_jar.get(OIDC_PKCE_COOKIE) else {
                return Err(AuthError::MissingCookie(OIDC_PKCE_COOKIE));
            };
            let Some(mut nonce_cookie) = private_jar.get(OIDC_NONCE_COOKIE) else {
                return Err(AuthError::MissingCookie(OIDC_NONCE_COOKIE));
            };
            let csrf_token = CsrfToken::new(csrf_cookie.value().to_string());
            let pkce_verifier = PkceCodeVerifier::new(pkce_cookie.value().to_string());
            let nonce = Nonce::new(nonce_cookie.value().to_string());
            csrf_cookie.make_removal();
            pkce_cookie.make_removal();
            nonce_cookie.make_removal();
            (csrf_token, pkce_verifier, nonce)
        };

        if &state != csrf_token.secret() {
            return Err(AuthError::InvalidState);
        }

        let token_response = self
            .oidc_client
            .exchange_code(AuthorizationCode::new(code))?
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.client)
            .await?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| AuthError::MissingIDToken)?;
        let claims = id_token.claims(&self.oidc_client.id_token_verifier(), &nonce)?;

        let provider_id = claims.subject().to_string();
        let display_name = claims
            .name()
            .and_then(|c| c.get(None))
            .map(|c| c.to_string())
            .ok_or_else(|| AuthError::MissingClaim("name"))?;

        let email = claims
            .email()
            .map(|e| e.to_string())
            .ok_or_else(|| AuthError::MissingClaim("email"))?;

        let user = User::get_by_external_id(txn, &self.config.name, &provider_id).await?;

        // first, see if we can attempt an account link via email (if enabled for this provider)
        let user = match user {
            None if self.config.auto_account_linking => {
                let user = User::get_by_email(txn, &email).await?;
                if let Some(user) = &user {
                    user.attach_external_id(txn, &self.config.name, &provider_id)
                        .await?;
                }
                user
            }
            user => user,
        };

        // otherwise, see if we can register them
        let user = match user {
            None if self.config.allow_registration => {
                let user = User::create(
                    txn,
                    UserCreation {
                        name: display_name,
                        email,
                    },
                )
                .await?;
                user.attach_external_id(txn, &self.config.name, &provider_id)
                    .await?;
                Some(user)
            }
            user => user,
        };

        let Some(user) = user else {
            return Err(AuthError::UserNotFound);
        };

        let session = user.create_session(txn).await?;

        let jar = jar.add({
            let mut cookie = Cookie::new(SESSION_COOKIE, session.to_string());
            cookie.set_http_only(true);
            cookie.set_same_site(SameSite::Lax);
            cookie.set_path("/");
            cookie
        });

        Ok((private_jar, jar, Redirect::to("/")))
    }
}
