use anyhow::Result;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    reqwest::async_http_client,
    AuthenticationFlow, ClientId as OidcClientId, ClientSecret as OidcClientSecret,
    IssuerUrl, Nonce, RedirectUrl as OidcRedirectUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub azure: Option<AzureConfig>,
    pub github: Option<GitHubConfig>,
    pub google: Option<GoogleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub provider: String,
}

pub struct OAuthService {
    config: OAuthConfig,
}

impl OAuthService {
    pub fn new(config: OAuthConfig) -> Self {
        Self { config }
    }

    pub fn get_azure_auth_url(&self) -> Result<(String, String)> {
        let azure_config = self.config.azure.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Azure OAuth not configured"))?;

        // For now, return a simple auth URL until we fix the OIDC client configuration
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email",
            azure_config.tenant_id,
            azure_config.client_id,
            urlencoding::encode(&azure_config.redirect_uri)
        );
        let csrf_token = "mock_csrf_token".to_string();

        Ok((auth_url, csrf_token))
    }

    pub fn get_github_auth_url(&self) -> Result<(String, String)> {
        let github_config = self.config.github.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub OAuth not configured"))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(github_config.client_id.clone()),
            Some(ClientSecret::new(github_config.client_secret.clone())),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?,
            Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(github_config.redirect_uri.clone())?);

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("user:email".to_string()))
            .add_scope(Scope::new("read:user".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    pub fn get_google_auth_url(&self) -> Result<(String, String)> {
        let google_config = self.config.google.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Google OAuth not configured"))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(google_config.client_id.clone()),
            Some(ClientSecret::new(google_config.client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://www.googleapis.com/oauth2/v4/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(google_config.redirect_uri.clone())?);

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .url();

        Ok((auth_url.to_string(), csrf_token.secret().clone()))
    }

    pub async fn handle_azure_callback(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<OAuthUser> {
        let azure_config = self.config.azure.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Azure OAuth not configured"))?;

        let issuer_url = IssuerUrl::new(format!(
            "https://login.microsoftonline.com/{}/v2.0",
            azure_config.tenant_id
        ))?;

        let auth_url = AuthUrl::new("https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string())?;
        let token_url = TokenUrl::new("https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string())?;

        let client = CoreClient::new(
            OidcClientId::new(azure_config.client_id.clone()),
            Some(OidcClientSecret::new(azure_config.client_secret.clone())),
            issuer_url,
            auth_url,
            Some(token_url),
            None, // UserInfoUrl
            openidconnect::JsonWebKeySet::new(vec![]), // Empty JsonWebKeySet
        );

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await?;

        // In a real implementation, you would verify the ID token and extract user info
        // For now, we'll create a mock user
        Ok(OAuthUser {
            id: "azure_user_id".to_string(),
            email: "user@company.com".to_string(),
            name: "Azure User".to_string(),
            avatar_url: None,
            provider: "azure".to_string(),
        })
    }

    pub async fn handle_github_callback(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<OAuthUser> {
        let github_config = self.config.github.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub OAuth not configured"))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(github_config.client_id.clone()),
            Some(ClientSecret::new(github_config.client_secret.clone())),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string())?,
            Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(github_config.redirect_uri.clone())?);

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await?;

        // Fetch user info from GitHub API
        let access_token = token_response.access_token().secret();
        let user_info = self.fetch_github_user_info(access_token).await?;

        Ok(user_info)
    }

    pub async fn handle_google_callback(
        &self,
        code: &str,
        _state: &str,
    ) -> Result<OAuthUser> {
        let google_config = self.config.google.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Google OAuth not configured"))?;

        let client = oauth2::basic::BasicClient::new(
            ClientId::new(google_config.client_id.clone()),
            Some(ClientSecret::new(google_config.client_secret.clone())),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
            Some(TokenUrl::new("https://www.googleapis.com/oauth2/v4/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(google_config.redirect_uri.clone())?);

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(async_http_client)
            .await?;

        // Fetch user info from Google API
        let access_token = token_response.access_token().secret();
        let user_info = self.fetch_google_user_info(access_token).await?;

        Ok(user_info)
    }

    async fn fetch_github_user_info(&self, access_token: &str) -> Result<OAuthUser> {
        let client = reqwest::Client::new();

        let user_response: GitHubUserResponse = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "Drift-Registry")
            .send()
            .await?
            .json()
            .await?;

        // Fetch primary email
        let emails_response: Vec<GitHubEmailResponse> = client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "Drift-Registry")
            .send()
            .await?
            .json()
            .await?;

        let primary_email = emails_response
            .iter()
            .find(|email| email.primary)
            .map(|email| email.email.clone())
            .unwrap_or_else(|| user_response.email.unwrap_or_default());

        Ok(OAuthUser {
            id: user_response.id.to_string(),
            email: primary_email,
            name: user_response.name.unwrap_or(user_response.login),
            avatar_url: Some(user_response.avatar_url),
            provider: "github".to_string(),
        })
    }

    async fn fetch_google_user_info(&self, access_token: &str) -> Result<OAuthUser> {
        let client = reqwest::Client::new();

        let user_response: GoogleUserResponse = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .json()
            .await?;

        Ok(OAuthUser {
            id: user_response.id,
            email: user_response.email,
            name: user_response.name,
            avatar_url: Some(user_response.picture),
            provider: "google".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct GitHubUserResponse {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubEmailResponse {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct GoogleUserResponse {
    id: String,
    email: String,
    name: String,
    picture: String,
    verified_email: bool,
}