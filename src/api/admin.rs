use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::garbage_collector::{GarbageCollector, GarbageCollectorMetrics};
use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbageCollectionRequest {
    pub dry_run: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbageCollectionResponse {
    pub success: bool,
    pub message: String,
    pub metrics: Option<GarbageCollectorMetrics>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/gc", post(trigger_garbage_collection))
        .route("/gc/status", get(get_gc_status))
}

async fn trigger_garbage_collection(
    State(state): State<AppState>,
    Json(request): Json<GarbageCollectionRequest>,
) -> impl IntoResponse {
    info!("Admin API: Triggering garbage collection");

    // Get garbage collector config
    let gc_config = match &state.config.garbage_collector {
        Some(config) => config.clone(),
        None => {
            return Json(GarbageCollectionResponse {
                success: false,
                message: "Garbage collection is not configured".to_string(),
                metrics: None,
            })
        }
    };

    // Override dry_run if specified in request
    let mut gc_config = gc_config;
    if let Some(dry_run) = request.dry_run {
        gc_config.dry_run = dry_run;
    }

    // Create garbage collector instance
    let gc = GarbageCollector::new(gc_config, state.storage.clone());

    // Run garbage collection
    match gc.trigger_manual_run().await {
        Ok(metrics) => {
            info!("Manual garbage collection completed successfully");
            Json(GarbageCollectionResponse {
                success: true,
                message: format!(
                    "Garbage collection completed: {} blobs deleted, {} manifests deleted, {} bytes freed",
                    metrics.blobs_deleted,
                    metrics.manifests_deleted,
                    metrics.bytes_freed
                ),
                metrics: Some(metrics),
            })
        }
        Err(e) => {
            error!("Manual garbage collection failed: {}", e);
            Json(GarbageCollectionResponse {
                success: false,
                message: format!("Garbage collection failed: {}", e),
                metrics: None,
            })
        }
    }
}

async fn get_gc_status(State(state): State<AppState>) -> impl IntoResponse {
    let gc_config = &state.config.garbage_collector;

    let response = match gc_config {
        Some(config) => serde_json::json!({
            "enabled": config.enabled,
            "interval_hours": config.interval_hours,
            "grace_period_hours": config.grace_period_hours,
            "dry_run": config.dry_run,
            "max_blobs_per_run": config.max_blobs_per_run,
            "status": "configured"
        }),
        None => serde_json::json!({
            "enabled": false,
            "status": "not_configured"
        }),
    };

    Json(response)
}

