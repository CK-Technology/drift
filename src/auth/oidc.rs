use anyhow::Result;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    ClientId as OidcClientId, ClientSecret as OidcClientSecret,
    IssuerUrl, Nonce, RedirectUrl as OidcRedirectUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub enabled: bool,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub provider: String,
}

pub struct OidcService {
    config: OidcConfig,
}

impl OidcService {
    pub fn new(config: OidcConfig) -> Self {
        Self { config }
    }

    pub async fn discover_metadata(&self) -> Result<CoreProviderMetadata> {
        let issuer_url = IssuerUrl::new(self.config.issuer_url.clone())?;
        let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client).await?;
        Ok(provider_metadata)
    }

    pub async fn create_client(&self) -> Result<CoreClient> {
        let provider_metadata = self.discover_metadata().await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            OidcClientId::new(self.config.client_id.clone()),
            Some(OidcClientSecret::new(self.config.client_secret.clone())),
        )
        .set_redirect_uri(OidcRedirectUrl::new(self.config.redirect_uri.clone())?);

        Ok(client)
    }
}