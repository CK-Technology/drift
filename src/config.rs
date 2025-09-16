use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::auth::oauth::{AzureConfig, GitHubConfig, GoogleConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub registry: RegistryConfig,
    pub bolt: Option<BoltConfig>,
    pub ghostbay: Option<GhostBayConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub ui_addr: String,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: StorageType,
    pub path: Option<String>,
    pub s3: Option<S3Config>,
    pub ghostbay: Option<GhostBayStorageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    Filesystem,
    S3,
    GhostBay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub path_style: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostBayStorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub credentials: Option<GhostBayCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostBayCredentials {
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub mode: AuthMode,
    pub jwt_secret: String,
    pub token_expiry_hours: u64,
    pub basic: Option<BasicAuthConfig>,
    pub oidc: Option<OidcConfig>,
    pub oauth: Option<OAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub enabled: bool,
    pub azure: Option<AzureConfig>,
    pub github: Option<GitHubConfig>,
    pub google: Option<GoogleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    Basic,
    Token,
    Oidc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub users: Vec<String>, // Format: "username:password"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub max_upload_size_mb: u64,
    pub rate_limit_per_hour: u32,
    pub immutable_tags: Vec<String>,
    pub min_age_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoltConfig {
    pub enable_profile_validation: bool,
    pub enable_plugin_sandbox: bool,
    pub auto_update_profiles: bool,
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostBayConfig {
    pub enable_s3_compat: bool,
    pub storage_engine: String,
    pub max_object_size_gb: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_addr: "0.0.0.0:5000".to_string(),
                ui_addr: "0.0.0.0:5001".to_string(),
                workers: Some(4),
                max_connections: Some(1000),
            },
            storage: StorageConfig {
                storage_type: StorageType::Filesystem,
                path: Some("./data".to_string()),
                s3: None,
                ghostbay: None,
            },
            auth: AuthConfig {
                mode: AuthMode::Basic,
                jwt_secret: "change-me-in-production".to_string(),
                token_expiry_hours: 24,
                basic: Some(BasicAuthConfig {
                    users: vec!["admin:changeme".to_string()],
                }),
                oidc: None,
                oauth: Some(OAuthConfig {
                    enabled: false,
                    azure: None,
                    github: None,
                    google: None,
                }),
            },
            registry: RegistryConfig {
                max_upload_size_mb: 1000,
                rate_limit_per_hour: 1000,
                immutable_tags: vec!["release".to_string(), "prod".to_string()],
                min_age_days: 7,
            },
            bolt: Some(BoltConfig {
                enable_profile_validation: true,
                enable_plugin_sandbox: true,
                auto_update_profiles: false,
                registry_url: None,
            }),
            ghostbay: Some(GhostBayConfig {
                enable_s3_compat: true,
                storage_engine: "local".to_string(),
                max_object_size_gb: 50,
            }),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}