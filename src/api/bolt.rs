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

use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub min_cpu_cores: Option<u32>,
    pub min_memory_gb: Option<u32>,
    pub required_gpu_vendor: Option<String>,
    pub min_gpu_memory_gb: Option<u32>,
    pub supported_os: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
    State(_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let per_page = params.get("per_page").and_then(|p| p.parse().ok()).unwrap_or(20);

    // Mock data for now - would integrate with actual Bolt registry
    let profiles = vec![
        BoltProfile {
            name: "steam-gaming".to_string(),
            description: "Optimized profile for Steam gaming".to_string(),
            version: "1.2.0".to_string(),
            author: "gaming-team".to_string(),
            tags: vec!["gaming".to_string(), "steam".to_string(), "nvidia".to_string()],
            compatible_games: vec!["Counter-Strike 2".to_string(), "Dota 2".to_string()],
            downloads: 15420,
            rating: 4.8,
            system_requirements: SystemRequirements {
                min_cpu_cores: Some(4),
                min_memory_gb: Some(8),
                required_gpu_vendor: Some("nvidia".to_string()),
                min_gpu_memory_gb: Some(4),
                supported_os: vec!["linux".to_string(), "windows".to_string()],
            },
        },
        BoltProfile {
            name: "competitive-fps".to_string(),
            description: "High-performance profile for competitive FPS games".to_string(),
            version: "2.1.0".to_string(),
            author: "esports-team".to_string(),
            tags: vec!["competitive".to_string(), "fps".to_string(), "low-latency".to_string()],
            compatible_games: vec!["Valorant".to_string(), "CS2".to_string(), "Overwatch 2".to_string()],
            downloads: 8930,
            rating: 4.9,
            system_requirements: SystemRequirements {
                min_cpu_cores: Some(6),
                min_memory_gb: Some(16),
                required_gpu_vendor: None,
                min_gpu_memory_gb: Some(6),
                supported_os: vec!["linux".to_string(), "windows".to_string()],
            },
        },
    ];

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
    State(_state): State<AppState>,
    Json(search): Json<ProfileSearchRequest>,
) -> impl IntoResponse {
    info!("Searching profiles: {:?}", search);

    // Mock search implementation - would integrate with actual Bolt registry
    let mut profiles = vec![
        BoltProfile {
            name: "steam-gaming".to_string(),
            description: "Optimized profile for Steam gaming".to_string(),
            version: "1.2.0".to_string(),
            author: "gaming-team".to_string(),
            tags: vec!["gaming".to_string(), "steam".to_string(), "nvidia".to_string()],
            compatible_games: vec!["Counter-Strike 2".to_string(), "Dota 2".to_string()],
            downloads: 15420,
            rating: 4.8,
            system_requirements: SystemRequirements {
                min_cpu_cores: Some(4),
                min_memory_gb: Some(8),
                required_gpu_vendor: Some("nvidia".to_string()),
                min_gpu_memory_gb: Some(4),
                supported_os: vec!["linux".to_string(), "windows".to_string()],
            },
        },
    ];

    // Apply basic filtering
    if let Some(query) = &search.query {
        profiles.retain(|p| p.name.contains(query) || p.description.contains(query));
    }

    if let Some(tags) = &search.tags {
        profiles.retain(|p| tags.iter().any(|tag| p.tags.contains(tag)));
    }

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
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Getting profile: {}", name);

    // Mock profile - would fetch from actual storage
    let profile = BoltProfile {
        name: name.clone(),
        description: format!("Gaming profile: {}", name),
        version: "1.0.0".to_string(),
        author: "drift-user".to_string(),
        tags: vec!["gaming".to_string()],
        compatible_games: vec!["Universal".to_string()],
        downloads: 100,
        rating: 4.5,
        system_requirements: SystemRequirements {
            min_cpu_cores: Some(4),
            min_memory_gb: Some(8),
            required_gpu_vendor: None,
            min_gpu_memory_gb: Some(2),
            supported_os: vec!["linux".to_string()],
        },
    };

    Json(profile)
}

pub async fn download_profile(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Downloading profile: {}", name);

    // Mock download - would return actual profile data
    let profile_data = format!(
        r#"
[profile]
name = "{}"
version = "1.0.0"
description = "Gaming optimization profile"

[optimizations]
cpu_affinity = true
gpu_scheduling = "high"
memory_management = "aggressive"

[games]
supported = ["*"]
"#,
        name
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/vnd.bolt.profile.v1+toml".parse().unwrap(),
    );
    headers.insert(
        "Content-Disposition",
        format!("attachment; filename=\"{}.toml\"", name).parse().unwrap(),
    );

    (headers, profile_data)
}

pub async fn upload_profile(
    State(_state): State<AppState>,
    Json(upload): Json<ProfileUploadRequest>,
) -> impl IntoResponse {
    info!("Uploading profile: {}", upload.profile.name);

    // Mock upload - would integrate with actual storage
    Json(json!({
        "message": "Profile uploaded successfully",
        "profile": upload.profile.name,
        "version": upload.profile.version
    }))
}

pub async fn delete_profile(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("Deleting profile: {}", name);

    // Mock deletion - would delete from actual storage
    (StatusCode::NO_CONTENT, ())
}

// Plugin endpoints (similar structure to profiles)
pub async fn list_plugins(State(_state): State<AppState>) -> impl IntoResponse {
    let plugins = vec![
        BoltPlugin {
            name: "nvidia-dlss-optimizer".to_string(),
            description: "DLSS optimization plugin for NVIDIA GPUs".to_string(),
            version: "1.0.0".to_string(),
            author: "nvidia-team".to_string(),
            plugin_type: "gpu-optimization".to_string(),
            supported_platforms: vec!["linux-x86_64".to_string(), "windows-x86_64".to_string()],
            downloads: 5420,
            rating: 4.7,
        },
    ];

    Json(SearchResponse {
        results: plugins,
        total: 1,
        page: 1,
        per_page: 20,
        pages: 1,
    })
}

pub async fn search_plugins(State(_state): State<AppState>) -> impl IntoResponse {
    Json(SearchResponse {
        results: Vec::<BoltPlugin>::new(),
        total: 0,
        page: 1,
        per_page: 20,
        pages: 0,
    })
}

pub async fn get_plugin(State(_state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    Json(BoltPlugin {
        name,
        description: "Mock plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "drift-user".to_string(),
        plugin_type: "optimization".to_string(),
        supported_platforms: vec!["linux".to_string()],
        downloads: 0,
        rating: 0.0,
    })
}

pub async fn download_plugin(State(_state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Plugin download not implemented")
}

pub async fn upload_plugin(State(_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Plugin upload not implemented")
}

pub async fn delete_plugin(State(_state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    (StatusCode::NO_CONTENT, ())
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