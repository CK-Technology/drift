use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::auth::oauth::{AzureConfig, GitHubConfig, GoogleConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub auth: AuthConfig,
    pub registry: RegistryConfig,
    pub garbage_collector: Option<GarbageCollectorConfig>,
    pub bolt: Option<BoltConfig>,
    pub ghostbay: Option<GhostBayConfig>,
    pub quic: Option<QuicConfig>,
    pub signing: Option<SigningConfig>,
    pub optimization: Option<OptimizationConfig>,
    pub rbac: Option<RbacConfig>,
    pub audit: Option<AuditConfig>,
    pub cluster: Option<ClusterConfig>,
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

impl Default for BoltConfig {
    fn default() -> Self {
        Self {
            enable_profile_validation: true,
            enable_plugin_sandbox: true,
            auto_update_profiles: false,
            registry_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectorConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub grace_period_hours: u64,
    pub dry_run: bool,
    pub max_blobs_per_run: usize,
}

impl Default for GarbageCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 24, // Run daily
            grace_period_hours: 168, // 7 days grace period
            dry_run: false,
            max_blobs_per_run: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostBayConfig {
    pub enable_s3_compat: bool,
    pub storage_engine: String,
    pub max_object_size_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    pub enabled: bool,
    pub backend: String, // "quinn", "quiche", "gquic", or "mock"
    pub bind_addr: std::net::SocketAddr,
    pub cert_path: String,
    pub key_path: String,
    pub cert_chain: Vec<u8>, // DER encoded certificate chain
    pub private_key: Vec<u8>, // DER encoded private key
    pub max_connections: usize,
    pub max_idle_timeout_ms: u64,
    pub keep_alive_interval_ms: u64,
    pub application_protocols: Vec<String>,
    pub enable_0rtt: bool,
    pub enable_early_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    pub enabled: bool,
    pub default_key_id: String,
    pub signature_formats: Vec<String>, // "cosign", "notary-v2", "simple", "in-toto"
    pub verification_policy: VerificationPolicyConfig,
    pub signing_keys: Vec<SigningKeyConfig>,
    pub verification_keys: Vec<VerificationKeyConfig>,
    pub trust_stores: Vec<TrustStoreConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicyConfig {
    pub require_signatures: bool,
    pub required_signatures_count: usize,
    pub allowed_signature_formats: Vec<String>,
    pub allowed_algorithms: Vec<String>,
    pub trust_stores: Vec<String>,
    pub require_certificate_chain: bool,
    pub allow_self_signed: bool,
    pub max_signature_age_hours: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyConfig {
    pub key_id: String,
    pub algorithm: crate::signing::SignatureAlgorithm,
    pub key_path: String,
    pub certificate_path: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeyConfig {
    pub key_id: String,
    pub algorithm: crate::signing::SignatureAlgorithm,
    pub public_key_path: String,
    pub certificate_path: Option<String>,
    pub trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStoreConfig {
    pub name: String,
    pub root_certificate_paths: Vec<String>,
    pub intermediate_certificate_paths: Vec<String>,
    pub crl_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub enabled: bool,
    pub background_optimization: bool,
    pub optimization_schedule_cron: Option<String>,
    pub enable_compression_optimization: bool,
    pub enable_layer_deduplication: bool,
    pub enable_layer_squashing: bool,
    pub enable_base_image_optimization: bool,
    pub preferred_compression: String, // "gzip", "zstd", "lz4", "brotli"
    pub min_layer_size_mb: u64,
    pub max_optimization_time_seconds: u64,
    pub preserve_original: bool,
    pub optimization_workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub enabled: bool,
    pub default_role: String,
    pub enable_organization_isolation: bool,
    pub enable_team_based_access: bool,
    pub enable_attribute_based_access: bool,
    pub cache_ttl_seconds: u64,
    pub audit_authorization_decisions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub min_severity: String, // "debug", "info", "warning", "error", "critical"
    pub buffer_size: usize,
    pub flush_interval_seconds: u64,
    pub enabled_event_types: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub file_export: Option<FileExportConfig>,
    pub webhook_export: Option<WebhookExportConfig>,
    pub elasticsearch_export: Option<ElasticsearchExportConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExportConfig {
    pub path: String,
    pub format: String, // "json", "jsonlines", "csv"
    pub rotation_size_mb: u64,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookExportConfig {
    pub url: String,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchExportConfig {
    pub url: String,
    pub index_prefix: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub node_id: String,
    pub bind_address: String,
    pub seed_nodes: Vec<String>,
    pub consensus_protocol: String, // "raft", "gossip"
    pub replication_factor: usize,
    pub consistency_level: crate::cluster::ConsistencyLevel,
    pub heartbeat_interval_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub health_check_timeout_seconds: u64,
    pub election_timeout_seconds: u64,
    pub load_balancing_strategy: String,
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
            garbage_collector: Some(GarbageCollectorConfig::default()),
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
            quic: Some(QuicConfig {
                enabled: false, // Disabled by default
                backend: "mock".to_string(), // Use mock by default
                bind_addr: "0.0.0.0:5443".parse().unwrap(),
                cert_path: "./certs/server.crt".to_string(),
                key_path: "./certs/server.key".to_string(),
                cert_chain: vec![], // Empty by default
                private_key: vec![], // Empty by default
                max_connections: 1000,
                max_idle_timeout_ms: 60000,
                keep_alive_interval_ms: 30000,
                application_protocols: vec!["drift-registry".to_string()],
                enable_0rtt: false,
                enable_early_data: false,
            }),
            signing: Some(SigningConfig {
                enabled: false, // Disabled by default
                default_key_id: "default".to_string(),
                signature_formats: vec!["cosign".to_string(), "simple".to_string()],
                verification_policy: VerificationPolicyConfig {
                    require_signatures: false,
                    required_signatures_count: 1,
                    allowed_signature_formats: vec!["cosign".to_string(), "notary-v2".to_string(), "simple".to_string()],
                    allowed_algorithms: vec!["ecdsa-p256-sha256".to_string(), "rsa-pss-sha256".to_string()],
                    trust_stores: vec!["default".to_string()],
                    require_certificate_chain: false,
                    allow_self_signed: true,
                    max_signature_age_hours: Some(24 * 30), // 30 days
                },
                signing_keys: vec![],
                verification_keys: vec![],
                trust_stores: vec![],
            }),
            optimization: Some(OptimizationConfig {
                enabled: false, // Disabled by default
                background_optimization: true,
                optimization_schedule_cron: Some("0 2 * * *".to_string()), // Daily at 2 AM
                enable_compression_optimization: true,
                enable_layer_deduplication: true,
                enable_layer_squashing: false, // Advanced feature
                enable_base_image_optimization: false, // Advanced feature
                preferred_compression: "gzip".to_string(),
                min_layer_size_mb: 10, // Don't optimize layers smaller than 10MB
                max_optimization_time_seconds: 300, // 5 minutes max per layer
                preserve_original: true,
                optimization_workers: 2,
            }),
            rbac: Some(RbacConfig {
                enabled: false, // Disabled by default
                default_role: "viewer".to_string(),
                enable_organization_isolation: true,
                enable_team_based_access: true,
                enable_attribute_based_access: false,
                cache_ttl_seconds: 300, // 5 minutes
                audit_authorization_decisions: true,
            }),
            audit: Some(AuditConfig {
                enabled: false, // Disabled by default
                min_severity: "info".to_string(),
                buffer_size: 1000,
                flush_interval_seconds: 60,
                enabled_event_types: vec![],
                exclude_patterns: vec![],
                file_export: Some(FileExportConfig {
                    path: "./logs/audit.jsonl".to_string(),
                    format: "jsonlines".to_string(),
                    rotation_size_mb: 100,
                    retention_days: 90,
                }),
                webhook_export: None,
                elasticsearch_export: None,
            }),
            cluster: Some(ClusterConfig {
                enabled: false, // Disabled by default
                node_id: "node-1".to_string(),
                bind_address: "0.0.0.0:7000".to_string(),
                seed_nodes: vec![],
                consensus_protocol: "raft".to_string(),
                replication_factor: 3,
                consistency_level: crate::cluster::ConsistencyLevel::Quorum,
                heartbeat_interval_seconds: 30,
                health_check_interval_seconds: 10,
                health_check_timeout_seconds: 60,
                election_timeout_seconds: 300,
                load_balancing_strategy: "round_robin".to_string(),
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