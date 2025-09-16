use super::RegistryError;
use crate::server::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    body::Bytes,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, error, info};
use uuid::Uuid;

pub async fn start_upload(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, RegistryError> {
    let upload_uuid = Uuid::new_v4().to_string();
    info!("Starting upload: {}/{}", name, upload_uuid);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LOCATION,
        format!("/v2/{}/blobs/uploads/{}", name, upload_uuid).parse().unwrap(),
    );
    headers.insert(
        "Docker-Upload-UUID",
        upload_uuid.parse().unwrap(),
    );
    headers.insert(
        "Range",
        "0-0".parse().unwrap(),
    );

    Ok((StatusCode::ACCEPTED, headers))
}

pub async fn upload_chunk(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, RegistryError> {
    debug!("Uploading chunk: {}/{} ({} bytes)", name, uuid, body.len());

    // Parse Content-Range header
    let range = if let Some(range_header) = headers.get("Content-Range") {
        parse_content_range(range_header.to_str().unwrap_or(""))
    } else {
        // If no range specified, assume starting at 0
        (0, body.len() as u64)
    };

    match state.storage.put_upload_chunk(&uuid, range, body).await {
        Ok(()) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::LOCATION,
                format!("/v2/{}/blobs/uploads/{}", name, uuid).parse().unwrap(),
            );
            response_headers.insert(
                "Docker-Upload-UUID",
                uuid.parse().unwrap(),
            );
            response_headers.insert(
                "Range",
                format!("0-{}", range.1 - 1).parse().unwrap(),
            );

            Ok((StatusCode::ACCEPTED, response_headers))
        }
        Err(e) => {
            error!("Failed to upload chunk {}: {}", uuid, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to upload chunk".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn complete_upload(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<impl IntoResponse, RegistryError> {
    let digest = params.get("digest")
        .ok_or_else(|| RegistryError {
            code: "DIGEST_INVALID".to_string(),
            message: "Digest parameter required".to_string(),
            detail: None,
        })?;

    info!("Completing upload: {}/{} -> {}", name, uuid, digest);

    // If there's a body, this is the final chunk
    if !body.is_empty() {
        // Calculate current size and append final chunk
        let range = (0, body.len() as u64); // This should be calculated properly
        if let Err(e) = state.storage.put_upload_chunk(&uuid, range, body).await {
            error!("Failed to upload final chunk {}: {}", uuid, e);
            return Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to upload final chunk".to_string(),
                detail: None,
            });
        }
    }

    // Complete the upload
    match state.storage.complete_upload(&uuid, digest).await {
        Ok(()) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                format!("/v2/{}/blobs/{}", name, digest).parse().unwrap(),
            );
            headers.insert(
                "Docker-Content-Digest",
                digest.parse().unwrap(),
            );

            Ok((StatusCode::CREATED, headers))
        }
        Err(e) => {
            error!("Failed to complete upload {}: {}", uuid, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to complete upload".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn get_upload_status(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    debug!("Getting upload status: {}/{}", name, uuid);

    match state.storage.get_upload_url(&uuid).await {
        Ok(Some(_)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::LOCATION,
                format!("/v2/{}/blobs/uploads/{}", name, uuid).parse().unwrap(),
            );
            headers.insert(
                "Docker-Upload-UUID",
                uuid.parse().unwrap(),
            );
            // TODO: Calculate actual range
            headers.insert(
                "Range",
                "0-0".parse().unwrap(),
            );

            Ok((StatusCode::NO_CONTENT, headers))
        }
        Ok(None) => Err(RegistryError {
            code: "BLOB_UPLOAD_UNKNOWN".to_string(),
            message: format!("Upload {} not found", uuid),
            detail: None,
        }),
        Err(e) => {
            error!("Failed to get upload status {}: {}", uuid, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to get upload status".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn cancel_upload(
    State(state): State<AppState>,
    Path((name, uuid)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Cancelling upload: {}/{}", name, uuid);

    match state.storage.cancel_upload(&uuid).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to cancel upload {}: {}", uuid, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to cancel upload".to_string(),
                detail: None,
            })
        }
    }
}

fn parse_content_range(range_str: &str) -> (u64, u64) {
    // Parse "bytes start-end/total" format
    if let Some(range_part) = range_str.strip_prefix("bytes ") {
        if let Some((range, _total)) = range_part.split_once('/') {
            if let Some((start, end)) = range.split_once('-') {
                if let (Ok(start), Ok(end)) = (start.parse::<u64>(), end.parse::<u64>()) {
                    return (start, end + 1);
                }
            }
        }
    }
    (0, 0)
}