use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{info, warn};

use crate::quic::{QuicMessage, QuicTransport, QuicTransportBackend};
use crate::server::AppState;

/// QUIC registry API endpoints for managing QUIC connections and testing
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/quic/status", get(get_quic_status))
        .route("/quic/ping/:addr", post(ping_quic_endpoint))
        .route("/quic/stats", get(get_quic_stats))
        .route("/quic/config", get(get_quic_config))
        .route("/quic/test/blob/:digest", post(test_quic_blob_request))
        .route("/quic/test/manifest/:reference", post(test_quic_manifest_request))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuicStatusResponse {
    pub enabled: bool,
    pub backend: String,
    pub bind_addr: String,
    pub active_connections: u64,
    pub supported_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuicPingRequest {
    pub target_addr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuicTestRequest {
    pub target_addr: String,
    pub timeout_ms: Option<u64>,
}

/// Get QUIC transport status
pub async fn get_quic_status(State(state): State<AppState>) -> impl IntoResponse {
    info!("Getting QUIC status");

    let quic_config = state.config.quic.as_ref();

    let response = match quic_config {
        Some(config) if config.enabled => {
            // Check if QUIC transport is available
            let stats = match state.quic.as_ref() {
                Some(quic) => quic.get_stats().await,
                None => {
                    return (StatusCode::SERVICE_UNAVAILABLE, "QUIC transport not initialized").into_response();
                }
            };

            let mut supported_features = vec!["ping".to_string(), "blob-transfer".to_string(), "manifest-transfer".to_string()];

            #[cfg(feature = "quinn-quic")]
            if config.backend == "quinn" {
                supported_features.push("quinn-backend".to_string());
                supported_features.push("0rtt".to_string());
            }

            #[cfg(feature = "quiche-quic")]
            if config.backend == "quiche" {
                supported_features.push("quiche-backend".to_string());
                supported_features.push("early-data".to_string());
            }

            #[cfg(feature = "gquic")]
            if config.backend == "gquic" {
                supported_features.push("gquic-backend".to_string());
                supported_features.push("custom-protocols".to_string());
            }

            QuicStatusResponse {
                enabled: true,
                backend: config.backend.clone(),
                bind_addr: config.bind_addr.to_string(),
                active_connections: stats.get("active_connections").copied().unwrap_or(0),
                supported_features,
            }
        }
        Some(_) => QuicStatusResponse {
            enabled: false,
            backend: "disabled".to_string(),
            bind_addr: "".to_string(),
            active_connections: 0,
            supported_features: vec![],
        },
        None => QuicStatusResponse {
            enabled: false,
            backend: "not-configured".to_string(),
            bind_addr: "".to_string(),
            active_connections: 0,
            supported_features: vec![],
        },
    };

    Json(response).into_response()
}

/// Ping a QUIC endpoint
pub async fn ping_quic_endpoint(
    State(state): State<AppState>,
    Path(addr): Path<String>,
) -> impl IntoResponse {
    info!("Pinging QUIC endpoint: {}", addr);

    let quic = match state.quic.as_ref() {
        Some(quic) => quic,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "QUIC transport not available").into_response();
        }
    };

    let target_addr: SocketAddr = match addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid address format").into_response();
        }
    };

    match quic.ping(target_addr).await {
        Ok(true) => Json(json!({
            "status": "success",
            "target": addr,
            "message": "QUIC ping successful"
        })).into_response(),
        Ok(false) => Json(json!({
            "status": "failed",
            "target": addr,
            "message": "QUIC ping failed - no response"
        })).into_response(),
        Err(e) => {
            warn!("QUIC ping error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("QUIC ping error: {}", e)).into_response()
        }
    }
}

/// Get QUIC transport statistics
pub async fn get_quic_stats(State(state): State<AppState>) -> impl IntoResponse {
    info!("Getting QUIC statistics");

    let quic = match state.quic.as_ref() {
        Some(quic) => quic,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "QUIC transport not available").into_response();
        }
    };

    let stats = quic.get_stats().await;
    Json(stats).into_response()
}

/// Get QUIC configuration (sanitized)
pub async fn get_quic_config(State(state): State<AppState>) -> impl IntoResponse {
    info!("Getting QUIC configuration");

    let config = match state.config.quic.as_ref() {
        Some(config) => config,
        None => {
            return (StatusCode::NOT_FOUND, "QUIC not configured").into_response();
        }
    };

    // Return sanitized configuration (no sensitive data)
    Json(json!({
        "enabled": config.enabled,
        "backend": config.backend,
        "bind_addr": config.bind_addr.to_string(),
        "max_connections": config.max_connections,
        "max_idle_timeout_ms": config.max_idle_timeout_ms,
        "keep_alive_interval_ms": config.keep_alive_interval_ms,
        "application_protocols": config.application_protocols,
        "enable_0rtt": config.enable_0rtt,
        "enable_early_data": config.enable_early_data,
        "cert_configured": !config.cert_path.is_empty(),
        "key_configured": !config.key_path.is_empty(),
    })).into_response()
}

/// Test QUIC blob request
pub async fn test_quic_blob_request(
    State(state): State<AppState>,
    Path(digest): Path<String>,
    Json(test_req): Json<QuicTestRequest>,
) -> impl IntoResponse {
    info!("Testing QUIC blob request for digest: {}", digest);

    let quic = match state.quic.as_ref() {
        Some(quic) => quic,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "QUIC transport not available").into_response();
        }
    };

    let target_addr: SocketAddr = match test_req.target_addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid address format").into_response();
        }
    };

    let message = QuicMessage::BlobRequest { digest: digest.clone() };

    match quic.send_message(target_addr, message).await {
        Ok(QuicMessage::BlobResponse { digest: resp_digest, content, metadata }) => {
            Json(json!({
                "status": "success",
                "digest": resp_digest,
                "has_content": content.is_some(),
                "content_size": content.as_ref().map(|c| c.len()).unwrap_or(0),
                "has_metadata": metadata.is_some(),
                "target": test_req.target_addr
            })).into_response()
        }
        Ok(QuicMessage::Error { code, message }) => {
            Json(json!({
                "status": "error",
                "error_code": code,
                "error_message": message,
                "target": test_req.target_addr
            })).into_response()
        }
        Ok(other) => {
            warn!("Unexpected response type: {:?}", other);
            Json(json!({
                "status": "unexpected_response",
                "response_type": format!("{:?}", other),
                "target": test_req.target_addr
            })).into_response()
        }
        Err(e) => {
            warn!("QUIC blob request error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("QUIC request error: {}", e)).into_response()
        }
    }
}

/// Test QUIC manifest request
pub async fn test_quic_manifest_request(
    State(state): State<AppState>,
    Path(reference): Path<String>,
    Json(test_req): Json<QuicTestRequest>,
) -> impl IntoResponse {
    info!("Testing QUIC manifest request for reference: {}", reference);

    let quic = match state.quic.as_ref() {
        Some(quic) => quic,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, "QUIC transport not available").into_response();
        }
    };

    let target_addr: SocketAddr = match test_req.target_addr.parse() {
        Ok(addr) => addr,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid address format").into_response();
        }
    };

    let message = QuicMessage::ManifestRequest { reference: reference.clone() };

    match quic.send_message(target_addr, message).await {
        Ok(QuicMessage::ManifestResponse { reference: resp_ref, content, content_type }) => {
            Json(json!({
                "status": "success",
                "reference": resp_ref,
                "has_content": content.is_some(),
                "content_size": content.as_ref().map(|c| c.len()).unwrap_or(0),
                "content_type": content_type,
                "target": test_req.target_addr
            })).into_response()
        }
        Ok(QuicMessage::Error { code, message }) => {
            Json(json!({
                "status": "error",
                "error_code": code,
                "error_message": message,
                "target": test_req.target_addr
            })).into_response()
        }
        Ok(other) => {
            warn!("Unexpected response type: {:?}", other);
            Json(json!({
                "status": "unexpected_response",
                "response_type": format!("{:?}", other),
                "target": test_req.target_addr
            })).into_response()
        }
        Err(e) => {
            warn!("QUIC manifest request error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("QUIC request error: {}", e)).into_response()
        }
    }
}

/// QUIC registry integration for high-performance blob and manifest transfers
pub struct QuicRegistryIntegration {
    transport: Arc<QuicTransport>,
}

impl QuicRegistryIntegration {
    pub fn new(transport: Arc<QuicTransport>) -> Self {
        Self { transport }
    }

    /// Upload blob via QUIC to a remote registry
    pub async fn upload_blob_quic(
        &self,
        target: SocketAddr,
        digest: String,
        content: Vec<u8>,
        metadata: crate::quic::BlobMetadata,
    ) -> Result<()> {
        let message = QuicMessage::BlobUpload {
            digest,
            content,
            metadata,
        };

        match self.transport.send_message(target, message).await? {
            QuicMessage::BlobResponse { .. } => Ok(()),
            QuicMessage::Error { code, message } => {
                anyhow::bail!("QUIC blob upload failed: {} ({})", message, code)
            }
            _ => anyhow::bail!("Unexpected response from QUIC blob upload"),
        }
    }

    /// Upload manifest via QUIC to a remote registry
    pub async fn upload_manifest_quic(
        &self,
        target: SocketAddr,
        reference: String,
        content: Vec<u8>,
        content_type: String,
    ) -> Result<()> {
        let message = QuicMessage::ManifestUpload {
            reference,
            content,
            content_type,
        };

        match self.transport.send_message(target, message).await? {
            QuicMessage::ManifestResponse { .. } => Ok(()),
            QuicMessage::Error { code, message } => {
                anyhow::bail!("QUIC manifest upload failed: {} ({})", message, code)
            }
            _ => anyhow::bail!("Unexpected response from QUIC manifest upload"),
        }
    }

    /// Download blob via QUIC from a remote registry
    pub async fn download_blob_quic(
        &self,
        target: SocketAddr,
        digest: String,
    ) -> Result<Option<(Vec<u8>, crate::quic::BlobMetadata)>> {
        let message = QuicMessage::BlobRequest { digest };

        match self.transport.send_message(target, message).await? {
            QuicMessage::BlobResponse { content: Some(content), metadata: Some(metadata), .. } => {
                Ok(Some((content, metadata)))
            }
            QuicMessage::BlobResponse { content: None, .. } => Ok(None),
            QuicMessage::Error { code, message } => {
                anyhow::bail!("QUIC blob download failed: {} ({})", message, code)
            }
            _ => anyhow::bail!("Unexpected response from QUIC blob download"),
        }
    }

    /// Download manifest via QUIC from a remote registry
    pub async fn download_manifest_quic(
        &self,
        target: SocketAddr,
        reference: String,
    ) -> Result<Option<(Vec<u8>, String)>> {
        let message = QuicMessage::ManifestRequest { reference };

        match self.transport.send_message(target, message).await? {
            QuicMessage::ManifestResponse { content: Some(content), content_type: Some(ct), .. } => {
                Ok(Some((content, ct)))
            }
            QuicMessage::ManifestResponse { content: None, .. } => Ok(None),
            QuicMessage::Error { code, message } => {
                anyhow::bail!("QUIC manifest download failed: {} ({})", message, code)
            }
            _ => anyhow::bail!("Unexpected response from QUIC manifest download"),
        }
    }
}

use std::sync::Arc;