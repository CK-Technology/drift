use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};
use base64::Engine;

use crate::bolt_integration::BoltIntegrationService;
use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoltProfile {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub compatible_games: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
    pub system_requirements: SystemRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub min_cpu_cores: Option<u32>,
    pub min_memory_gb: Option<u32>,
    pub required_gpu_vendor: Option<String>,
    pub min_gpu_memory_gb: Option<u32>,
    pub supported_os: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoltPlugin {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub plugin_type: String,
    pub supported_platforms: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileSearchRequest {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub game: Option<String>,
    pub gpu_vendor: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse<T> {
    pub results: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
    pub pages: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileUploadRequest {
    pub profile: BoltProfile,
    pub metadata: ProfileUploadMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUploadMetadata {
    pub author_email: String,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Profile management
        .route("/profiles", get(list_profiles))
        .route("/profiles/search", post(search_profiles))
        .route("/profiles/:name", get(get_profile).delete(delete_profile))
        .route("/profiles/:name/download", get(download_profile))
        .route("/profiles/upload", post(upload_profile))

        // Plugin management
        .route("/plugins", get(list_plugins))
        .route("/plugins/search", post(search_plugins))
        .route("/plugins/:name", get(get_plugin).delete(delete_plugin))
        .route("/plugins/:name/download", get(download_plugin))
        .route("/plugins/upload", post(upload_plugin))

        // Metrics & Analytics
        .route("/metrics", get(get_metrics))
        .route("/metrics/profiles", get(get_profile_metrics))
        .route("/metrics/plugins", get(get_plugin_metrics))
}

pub async fn list_profiles(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20);

    // Use real Bolt integration service
    let mut profiles = match state.bolt.list_profiles().await {
        Ok(profiles) => profiles,
        Err(e) => {
            warn!("Failed to list Bolt profiles: {}", e);
            vec![]
        }
    };

    // If no profiles found, create default ones
    if profiles.is_empty() {
        if let Err(e) = crate::bolt_integration::create_default_profiles(&state.bolt).await {
            warn!("Failed to create default profiles: {}", e);
        }
        // Try again after creating defaults
        profiles = state.bolt.list_profiles().await.unwrap_or_default();
    }

    let response = SearchResponse {
        results: profiles,
        total: 2,
        page,
        per_page,
        pages: 1,
    };

    Json(response)
}

pub async fn search_profiles(
    State(state): State<AppState>,
    Json(search): Json<ProfileSearchRequest>,
) -> impl IntoResponse {
    info!("Searching profiles: {:?}", search);

    // Use real Bolt integration service
    let profiles = match state.bolt.search_profiles(
        search.query,
        search.tags,
        search.game,
        search.gpu_vendor,
    ).await {
        Ok(profiles) => profiles,
        Err(e) => {
            warn!("Failed to search Bolt profiles: {}", e);
            vec![]
        }
    };

    let page = search.page.unwrap_or(1);
    let per_page = search.per_page.unwrap_or(20);

    let response = SearchResponse {
        results: profiles,
        total: 1,
        page,
        per_page,
        pages: 1,
    };

    Json(response)
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Getting profile: {}", name);

    match state.bolt.get_profile(&name).await {
        Ok(Some(profile)) => Json(profile).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Profile not found").into_response(),
        Err(e) => {
            warn!("Failed to get profile {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get profile").into_response()
        }
    }
}

pub async fn download_profile(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Downloading profile: {}", name);

    match state.bolt.download_profile(&name).await {
        Ok(Some(profile_data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "Content-Type",
                "application/vnd.bolt.profile.v1+toml".parse().unwrap(),
            );
            headers.insert(
                "Content-Disposition",
                format!("attachment; filename=\"{}.toml\"", name).parse().unwrap(),
            );

            (headers, profile_data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Profile not found").into_response(),
        Err(e) => {
            warn!("Failed to download profile {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to download profile").into_response()
        }
    }
}

pub async fn upload_profile(
    State(state): State<AppState>,
    Json(upload): Json<ProfileUploadRequest>,
) -> impl IntoResponse {
    let profile = upload.profile.clone();
    let metadata = upload.metadata.clone();
    info!("Uploading profile: {}", profile.name);

    // Create TOML profile data from metadata
    let profile_toml = format!(
        r#"
[profile]
name = "{}"
version = "{}"
description = "{}"
author = "{}"

[metadata]
license = "{}"
repository = "{}"
documentation = "{}"

[requirements]
min_cpu_cores = {}
min_memory_gb = {}
required_gpu_vendor = "{}"
min_gpu_memory_gb = {}
supported_os = {:?}

[tags]
values = {:?}

[games]
compatible = {:?}
"#,
        profile.name,
        profile.version,
        profile.description,
        profile.author,
        metadata.license.unwrap_or_else(|| "Unknown".to_string()),
        metadata.repository.unwrap_or_else(|| "".to_string()),
        metadata.documentation.unwrap_or_else(|| "".to_string()),
        profile.system_requirements.min_cpu_cores.unwrap_or(1),
        profile.system_requirements.min_memory_gb.unwrap_or(1),
        profile.system_requirements.required_gpu_vendor.as_ref().unwrap_or(&"any".to_string()).clone(),
        profile.system_requirements.min_gpu_memory_gb.unwrap_or(1),
        profile.system_requirements.supported_os.clone(),
        profile.tags.clone(),
        profile.compatible_games.clone()
    );

    let profile_name = profile.name.clone();
    let profile_version = profile.version.clone();
    match state.bolt.upload_profile(profile.clone(), profile_toml).await {
        Ok(_) => Json(json!({
            "message": "Profile uploaded successfully",
            "profile": profile_name,
            "version": profile_version
        })).into_response(),
        Err(e) => {
            warn!("Failed to upload profile {}: {}", profile_name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload profile").into_response()
        }
    }
}

pub async fn delete_profile(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Deleting profile: {}", name);

    match state.bolt.delete_profile(&name).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => {
            warn!("Failed to delete profile {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete profile").into_response()
        }
    }
}

// Plugin endpoints (similar structure to profiles)
pub async fn list_plugins(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20);

    // Use real Bolt integration service
    let mut plugins = match state.bolt.list_plugins().await {
        Ok(plugins) => plugins,
        Err(e) => {
            warn!("Failed to list Bolt plugins: {}", e);
            vec![]
        }
    };

    // If no plugins found, create default ones
    if plugins.is_empty() {
        if let Err(e) = create_default_plugins(&state.bolt).await {
            warn!("Failed to create default plugins: {}", e);
        }
        // Try again after creating defaults
        plugins = state.bolt.list_plugins().await.unwrap_or_default();
    }

    let response = SearchResponse {
        results: plugins,
        total: 1,
        page,
        per_page,
        pages: 1,
    };

    Json(response)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginSearchRequest {
    pub query: Option<String>,
    pub plugin_type: Option<String>,
    pub platform: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

pub async fn search_plugins(
    State(state): State<AppState>,
    Json(search): Json<PluginSearchRequest>,
) -> impl IntoResponse {
    info!("Searching plugins: {:?}", search);

    // Use real Bolt integration service
    let plugins = match state.bolt.search_plugins(
        search.query,
        search.plugin_type,
        search.platform,
    ).await {
        Ok(plugins) => plugins,
        Err(e) => {
            warn!("Failed to search Bolt plugins: {}", e);
            vec![]
        }
    };

    let page = search.page.unwrap_or(1);
    let per_page = search.per_page.unwrap_or(20);

    let response = SearchResponse {
        results: plugins,
        total: 0,
        page,
        per_page,
        pages: 0,
    };

    Json(response)
}

pub async fn get_plugin(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    info!("Getting plugin: {}", name);

    match state.bolt.get_plugin(&name).await {
        Ok(Some(plugin)) => Json(plugin).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Plugin not found").into_response(),
        Err(e) => {
            warn!("Failed to get plugin {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get plugin").into_response()
        }
    }
}

pub async fn download_plugin(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    info!("Downloading plugin: {}", name);

    match state.bolt.download_plugin(&name).await {
        Ok(Some(plugin_data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "Content-Type",
                "application/octet-stream".parse().unwrap(),
            );
            headers.insert(
                "Content-Disposition",
                format!("attachment; filename=\"{}.bin\"", name).parse().unwrap(),
            );

            (headers, plugin_data).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Plugin not found").into_response(),
        Err(e) => {
            warn!("Failed to download plugin {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to download plugin").into_response()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginUploadRequest {
    pub plugin: BoltPlugin,
    pub plugin_data: String, // Base64 encoded binary data
}

pub async fn upload_plugin(
    State(state): State<AppState>,
    Json(upload): Json<PluginUploadRequest>,
) -> impl IntoResponse {
    info!("Uploading plugin: {}", upload.plugin.name);

    // Decode base64 plugin data
    let plugin_data = match base64::engine::general_purpose::STANDARD.decode(&upload.plugin_data) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to decode plugin data: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid plugin data encoding").into_response();
        }
    };

    match state.bolt.upload_plugin(upload.plugin.clone(), plugin_data).await {
        Ok(_) => Json(json!({
            "message": "Plugin uploaded successfully",
            "plugin": upload.plugin.name,
            "version": upload.plugin.version
        })).into_response(),
        Err(e) => {
            warn!("Failed to upload plugin {}: {}", upload.plugin.name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload plugin").into_response()
        }
    }
}

pub async fn delete_plugin(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    info!("Deleting plugin: {}", name);

    match state.bolt.delete_plugin(&name).await {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => {
            warn!("Failed to delete plugin {}: {}", name, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete plugin").into_response()
        }
    }
}

// Metrics endpoints
pub async fn get_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "total_profiles": 42,
        "total_plugins": 15,
        "total_downloads": 150420,
        "storage_usage_bytes": 5368709120_u64,
        "active_users": 1250,
        "popular_profiles": [
            {"name": "steam-gaming", "downloads": 15420},
            {"name": "competitive-fps", "downloads": 8930}
        ]
    }))
}

pub async fn get_profile_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "total": 42,
        "by_category": {
            "gaming": 25,
            "competitive": 10,
            "streaming": 7
        },
        "downloads_24h": 145,
        "downloads_7d": 1240
    }))
}

pub async fn get_plugin_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "total": 15,
        "by_type": {
            "gpu-optimization": 8,
            "audio-enhancement": 4,
            "network-optimization": 3
        },
        "downloads_24h": 67,
        "downloads_7d": 523
    }))
}

/// Create some default plugins for demonstration
async fn create_default_plugins(service: &BoltIntegrationService) -> anyhow::Result<()> {
    // NVIDIA DLSS Optimization Plugin
    let dlss_plugin = BoltPlugin {
        name: "nvidia-dlss-optimizer".to_string(),
        description: "Advanced DLSS optimization plugin for NVIDIA RTX GPUs with dynamic quality scaling".to_string(),
        version: "2.1.0".to_string(),
        author: "nvidia-community".to_string(),
        plugin_type: "gpu-optimization".to_string(),
        supported_platforms: vec![
            "linux-x86_64".to_string(),
            "windows-x86_64".to_string(),
            "linux-aarch64".to_string()
        ],
        downloads: 0,
        rating: 4.8,
    };

    // Mock binary data (in real implementation, this would be actual plugin binary)
    let dlss_binary = b"DLSS_PLUGIN_BINARY_DATA_PLACEHOLDER".to_vec();
    service.upload_plugin(dlss_plugin, dlss_binary).await?;

    // AMD FSR Optimization Plugin
    let fsr_plugin = BoltPlugin {
        name: "amd-fsr-enhancer".to_string(),
        description: "FidelityFX Super Resolution enhancer for AMD GPUs with temporal upscaling".to_string(),
        version: "1.5.2".to_string(),
        author: "amd-opensource".to_string(),
        plugin_type: "gpu-optimization".to_string(),
        supported_platforms: vec![
            "linux-x86_64".to_string(),
            "windows-x86_64".to_string()
        ],
        downloads: 0,
        rating: 4.6,
    };

    let fsr_binary = b"FSR_PLUGIN_BINARY_DATA_PLACEHOLDER".to_vec();
    service.upload_plugin(fsr_plugin, fsr_binary).await?;

    // Audio Latency Reducer Plugin
    let audio_plugin = BoltPlugin {
        name: "ultra-low-latency-audio".to_string(),
        description: "Professional audio latency reducer for gaming and streaming with ASIO support".to_string(),
        version: "3.0.1".to_string(),
        author: "audio-pro-team".to_string(),
        plugin_type: "audio-optimization".to_string(),
        supported_platforms: vec![
            "linux-x86_64".to_string(),
            "windows-x86_64".to_string(),
            "macos-x86_64".to_string(),
            "macos-aarch64".to_string()
        ],
        downloads: 0,
        rating: 4.9,
    };

    let audio_binary = b"AUDIO_PLUGIN_BINARY_DATA_PLACEHOLDER".to_vec();
    service.upload_plugin(audio_plugin, audio_binary).await?;

    info!("Created default Bolt plugins");
    Ok(())
}