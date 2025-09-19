use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::AuditConfig;
use crate::storage::StorageBackend;

/// Comprehensive audit logging system for drift registry
#[derive(Clone)]
pub struct AuditService {
    config: AuditConfig,
    storage: Arc<dyn StorageBackend>,
    buffer: Arc<RwLock<Vec<AuditEvent>>>,
    exporters: Arc<RwLock<Vec<Box<dyn AuditExporter>>>>,
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: EventType,
    pub severity: Severity,
    pub user: UserInfo,
    pub resource: ResourceInfo,
    pub action: ActionInfo,
    pub result: EventResult,
    pub network: NetworkInfo,
    pub metadata: HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    // Authentication events
    Login,
    Logout,
    TokenIssued,
    TokenRefreshed,
    TokenRevoked,
    AuthenticationFailed,

    // Authorization events
    PermissionGranted,
    PermissionDenied,
    RoleAssigned,
    RoleRevoked,

    // Registry operations
    ImagePulled,
    ImagePushed,
    ImageDeleted,
    ManifestCreated,
    ManifestDeleted,
    BlobUploaded,
    BlobDeleted,

    // Administrative events
    UserCreated,
    UserModified,
    UserDeleted,
    OrganizationCreated,
    OrganizationModified,
    OrganizationDeleted,
    TeamCreated,
    TeamModified,
    TeamDeleted,

    // Security events
    SignatureCreated,
    SignatureVerified,
    SignatureInvalid,
    QuotaExceeded,
    RateLimitExceeded,
    SuspiciousActivity,

    // System events
    ConfigurationChanged,
    ServiceStarted,
    ServiceStopped,
    BackupCreated,
    RestoreCompleted,
    GarbageCollectionRun,
    OptimizationRun,

    // Custom events
    Custom(String),
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// User information in audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub teams: Vec<String>,
    pub roles: Vec<String>,
    pub service_account: bool,
}

/// Resource information in audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub type_: String,
    pub id: String,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub repository: Option<String>,
    pub tag: Option<String>,
    pub digest: Option<String>,
    pub size: Option<u64>,
}

/// Action information in audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub operation: String,
    pub method: Option<String>, // HTTP method
    pub path: Option<String>,    // API path
    pub parameters: HashMap<String, String>,
}

/// Event result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Network information in audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub client_ip: Option<String>,
    pub client_port: Option<u16>,
    pub server_ip: Option<String>,
    pub server_port: Option<u16>,
    pub protocol: Option<String>, // HTTP, HTTPS, QUIC
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

/// Trait for audit event exporters
#[async_trait]
pub trait AuditExporter: Send + Sync {
    async fn export(&self, events: &[AuditEvent]) -> Result<()>;
    fn name(&self) -> String;
}

/// File-based audit exporter
pub struct FileExporter {
    path: String,
    format: ExportFormat,
}

/// Export formats
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    JsonLines,
    Csv,
    Syslog,
}

/// Webhook audit exporter
pub struct WebhookExporter {
    url: String,
    headers: HashMap<String, String>,
    timeout_seconds: u64,
}

/// Elasticsearch audit exporter
pub struct ElasticsearchExporter {
    url: String,
    index_prefix: String,
    username: Option<String>,
    password: Option<String>,
}

/// Audit query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub event_types: Vec<EventType>,
    pub severities: Vec<Severity>,
    pub user_id: Option<String>,
    pub organization: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub success_only: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_severity: HashMap<String, u64>,
    pub failed_events: u64,
    pub avg_duration_ms: f64,
    pub top_users: Vec<(String, u64)>,
    pub top_resources: Vec<(String, u64)>,
}

impl AuditService {
    pub async fn new(
        config: AuditConfig,
        storage: Arc<dyn StorageBackend>,
    ) -> Result<Self> {
        info!("Initializing audit service");

        let service = Self {
            config,
            storage,
            buffer: Arc::new(RwLock::new(Vec::new())),
            exporters: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize exporters based on configuration
        service.initialize_exporters().await?;

        // Start background flushing task
        service.start_flush_task();

        info!("Audit service initialized successfully");
        Ok(service)
    }

    /// Initialize configured exporters
    async fn initialize_exporters(&self) -> Result<()> {
        let mut exporters = self.exporters.write().await;

        // File exporter
        if let Some(file_config) = &self.config.file_export {
            exporters.push(Box::new(FileExporter {
                path: file_config.path.clone(),
                format: ExportFormat::JsonLines,
            }));
        }

        // Webhook exporter
        if let Some(webhook_config) = &self.config.webhook_export {
            exporters.push(Box::new(WebhookExporter {
                url: webhook_config.url.clone(),
                headers: webhook_config.headers.clone(),
                timeout_seconds: webhook_config.timeout_seconds,
            }));
        }

        // Elasticsearch exporter
        if let Some(es_config) = &self.config.elasticsearch_export {
            exporters.push(Box::new(ElasticsearchExporter {
                url: es_config.url.clone(),
                index_prefix: es_config.index_prefix.clone(),
                username: es_config.username.clone(),
                password: es_config.password.clone(),
            }));
        }

        info!("Initialized {} audit exporters", exporters.len());
        Ok(())
    }

    /// Start background task to flush audit buffer
    fn start_flush_task(&self) {
        let buffer = self.buffer.clone();
        let exporters = self.exporters.clone();
        let storage = self.storage.clone();
        let flush_interval = self.config.flush_interval_seconds;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(flush_interval)).await;

                let events = {
                    let mut buf = buffer.write().await;
                    std::mem::take(&mut *buf)
                };

                if !events.is_empty() {
                    debug!("Flushing {} audit events", events.len());

                    // Export to all configured exporters
                    let exporters = exporters.read().await;
                    for exporter in exporters.iter() {
                        if let Err(e) = exporter.export(&events).await {
                            error!("Failed to export audit events via {}: {}", exporter.name(), e);
                        }
                    }

                    // Store in primary storage
                    if let Err(e) = Self::store_events(&storage, &events).await {
                        error!("Failed to store audit events: {}", e);
                    }
                }
            }
        });
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        debug!("Logging audit event: {:?}", event.event_type);

        // Apply filtering rules
        if !self.should_log(&event) {
            return Ok(());
        }

        // Add to buffer
        let mut buffer = self.buffer.write().await;
        buffer.push(event.clone());

        // Check if immediate flush is needed
        if buffer.len() >= self.config.buffer_size || event.severity >= Severity::Error {
            let events = std::mem::take(&mut *buffer);
            drop(buffer); // Release lock

            // Export immediately for critical events
            let exporters = self.exporters.read().await;
            for exporter in exporters.iter() {
                if let Err(e) = exporter.export(&events).await {
                    error!("Failed to export critical audit event: {}", e);
                }
            }

            // Store immediately
            Self::store_events(&self.storage, &events).await?;
        }

        Ok(())
    }

    /// Check if event should be logged based on configuration
    fn should_log(&self, event: &AuditEvent) -> bool {
        // Check severity threshold
        let min_severity = match self.config.min_severity.as_str() {
            "debug" => Severity::Debug,
            "info" => Severity::Info,
            "warning" => Severity::Warning,
            "error" => Severity::Error,
            "critical" => Severity::Critical,
            _ => Severity::Info,
        };
        if event.severity < min_severity {
            return false;
        }

        // Check event type filters
        if !self.config.enabled_event_types.is_empty() {
            if !self.config.enabled_event_types.contains(&format!("{:?}", event.event_type)) {
                return false;
            }
        }

        // Check exclusion patterns
        for pattern in &self.config.exclude_patterns {
            if event.resource.id.contains(pattern) ||
               event.user.username.as_ref().map_or(false, |u| u.contains(pattern)) {
                return false;
            }
        }

        true
    }

    /// Store events in primary storage
    async fn store_events(storage: &Arc<dyn StorageBackend>, events: &[AuditEvent]) -> Result<()> {
        for event in events {
            let key = format!("audit/{}/{}/{}.json",
                event.timestamp.format("%Y/%m/%d"),
                event.event_type.to_string().to_lowercase(),
                event.id
            );

            let data = serde_json::to_vec(event)?;
            storage.put_blob(&key, data.into()).await?;
        }

        Ok(())
    }

    /// Query audit events
    pub async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEvent>> {
        debug!("Querying audit events: {:?}", query);

        let mut events = Vec::new();
        let mut count = 0;
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        // Build date range for scanning
        let start = query.start_time.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(7));
        let end = query.end_time.unwrap_or_else(|| chrono::Utc::now());

        // Scan storage for matching events
        let mut current = start.date_naive();
        while current <= end.date_naive() {
            let prefix = format!("audit/{}/", current.format("%Y/%m/%d"));

            // In real implementation, would list and filter blobs
            debug!("Scanning audit events for date: {}", current);

            current = current.succ_opt().unwrap_or(current);
        }

        Ok(events)
    }

    /// Get audit statistics
    pub async fn get_stats(&self, duration_hours: u64) -> AuditStats {
        let since = chrono::Utc::now() - chrono::Duration::hours(duration_hours as i64);

        // In real implementation, would calculate from stored events
        AuditStats {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            failed_events: 0,
            avg_duration_ms: 0.0,
            top_users: vec![],
            top_resources: vec![],
        }
    }

    /// Create standard audit event builders
    pub fn login_event(user: UserInfo, success: bool, ip: Option<String>) -> AuditEvent {
        AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: if success { EventType::Login } else { EventType::AuthenticationFailed },
            severity: if success { Severity::Info } else { Severity::Warning },
            user,
            resource: ResourceInfo {
                type_: "auth".to_string(),
                id: "login".to_string(),
                name: None,
                namespace: None,
                repository: None,
                tag: None,
                digest: None,
                size: None,
            },
            action: ActionInfo {
                operation: "login".to_string(),
                method: Some("POST".to_string()),
                path: Some("/auth/login".to_string()),
                parameters: HashMap::new(),
            },
            result: EventResult {
                success,
                status_code: Some(if success { 200 } else { 401 }),
                error_message: if !success { Some("Authentication failed".to_string()) } else { None },
                error_code: None,
                duration_ms: None,
            },
            network: NetworkInfo {
                client_ip: ip,
                client_port: None,
                server_ip: None,
                server_port: None,
                protocol: Some("HTTPS".to_string()),
                user_agent: None,
                request_id: None,
            },
            metadata: HashMap::new(),
            correlation_id: None,
        }
    }

    pub fn image_pull_event(user: UserInfo, repository: String, tag: String, digest: String, success: bool) -> AuditEvent {
        AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: EventType::ImagePulled,
            severity: Severity::Info,
            user,
            resource: ResourceInfo {
                type_: "image".to_string(),
                id: format!("{}:{}", repository, tag),
                name: Some(repository.clone()),
                namespace: None,
                repository: Some(repository),
                tag: Some(tag),
                digest: Some(digest),
                size: None,
            },
            action: ActionInfo {
                operation: "pull".to_string(),
                method: Some("GET".to_string()),
                path: Some("/v2/{name}/manifests/{reference}".to_string()),
                parameters: HashMap::new(),
            },
            result: EventResult {
                success,
                status_code: Some(if success { 200 } else { 404 }),
                error_message: None,
                error_code: None,
                duration_ms: None,
            },
            network: NetworkInfo {
                client_ip: None,
                client_port: None,
                server_ip: None,
                server_port: None,
                protocol: Some("HTTPS".to_string()),
                user_agent: None,
                request_id: None,
            },
            metadata: HashMap::new(),
            correlation_id: None,
        }
    }
}

#[async_trait]
impl AuditExporter for FileExporter {
    async fn export(&self, events: &[AuditEvent]) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        for event in events {
            let line = match self.format {
                ExportFormat::JsonLines => {
                    let mut json = serde_json::to_string(event)?;
                    json.push('\n');
                    json
                }
                _ => {
                    // Other formats not implemented yet
                    continue;
                }
            };

            file.write_all(line.as_bytes()).await?;
        }

        file.flush().await?;
        Ok(())
    }

    fn name(&self) -> String {
        format!("FileExporter({})", self.path)
    }
}

#[async_trait]
impl AuditExporter for WebhookExporter {
    async fn export(&self, events: &[AuditEvent]) -> Result<()> {
        let client = reqwest::Client::new();

        let response = client
            .post(&self.url)
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .json(events)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Webhook export failed: {}", response.status()));
        }

        Ok(())
    }

    fn name(&self) -> String {
        format!("WebhookExporter({})", self.url)
    }
}

#[async_trait]
impl AuditExporter for ElasticsearchExporter {
    async fn export(&self, events: &[AuditEvent]) -> Result<()> {
        // Simplified Elasticsearch export
        warn!("Elasticsearch export not fully implemented");
        Ok(())
    }

    fn name(&self) -> String {
        format!("ElasticsearchExporter({})", self.url)
    }
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}