use axum::{
    extract::{Path, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, head, patch, post, put},
    Json, Router,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

use crate::server::AppState;

pub mod blobs;
pub mod manifests;
pub mod uploads;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryList {
    pub repositories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagList {
    pub name: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryError {
    pub code: String,
    pub message: String,
    pub detail: Option<serde_json::Value>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        // Registry API version check
        .route("/", get(api_version))

        // Repository catalog
        .route("/_catalog", get(list_repositories))

        // Manifest operations
        .route(
            "/:name/manifests/:reference",
            get(manifests::get_manifest)
                .put(manifests::put_manifest)
                .delete(manifests::delete_manifest)
                .head(manifests::head_manifest),
        )

        // Blob operations
        .route(
            "/:name/blobs/:digest",
            get(blobs::get_blob)
                .head(blobs::head_blob)
                .delete(blobs::delete_blob),
        )

        // Upload operations
        .route("/:name/blobs/uploads/", post(uploads::start_upload))
        .route(
            "/:name/blobs/uploads/:uuid",
            patch(uploads::upload_chunk)
                .put(uploads::complete_upload)
                .get(uploads::get_upload_status)
                .delete(uploads::cancel_upload),
        )

        // Tag listing
        .route("/:name/tags/list", get(list_tags))
}

pub async fn api_version() -> impl IntoResponse {
    Json(json!({
        "name": "drift",
        "version": "0.1.0",
        "description": "Drift OCI Registry",
        "api_version": "registry/2.0"
    }))
}

pub async fn list_repositories(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, RegistryError> {
    let n = params
        .get("n")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    let last = params.get("last");

    match state.storage.list_repositories().await {
        Ok(mut repos) => {
            // Apply pagination
            if let Some(last_repo) = last {
                if let Some(pos) = repos.iter().position(|r| r > last_repo) {
                    repos = repos.into_iter().skip(pos).collect();
                }
            }

            repos.truncate(n);

            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );

            let response = RepositoryList { repositories: repos };
            Ok((headers, Json(response)))
        }
        Err(e) => {
            error!("Failed to list repositories: {}", e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to list repositories".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn list_tags(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, RegistryError> {
    let n = params
        .get("n")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);

    let last = params.get("last");

    match state.storage.list_tags(&name).await {
        Ok(mut tags) => {
            // Apply pagination
            if let Some(last_tag) = last {
                if let Some(pos) = tags.iter().position(|t| t > last_tag) {
                    tags = tags.into_iter().skip(pos).collect();
                }
            }

            tags.truncate(n);

            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );

            let response = TagList { name, tags };
            Ok((headers, Json(response)))
        }
        Err(e) => {
            error!("Failed to list tags for repository {}: {}", name, e);
            Err(RegistryError {
                code: "NAME_UNKNOWN".to_string(),
                message: format!("Repository {} not found", name),
                detail: None,
            })
        }
    }
}

impl IntoResponse for RegistryError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "NAME_UNKNOWN" => StatusCode::NOT_FOUND,
            "MANIFEST_UNKNOWN" => StatusCode::NOT_FOUND,
            "BLOB_UNKNOWN" => StatusCode::NOT_FOUND,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "DENIED" => StatusCode::FORBIDDEN,
            "UNSUPPORTED" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = json!({
            "errors": [self]
        });

        (status, Json(body)).into_response()
    }
}