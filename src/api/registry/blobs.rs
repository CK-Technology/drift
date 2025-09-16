use super::RegistryError;
use crate::server::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use tracing::{debug, error, info};

pub async fn get_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Getting blob: {}/{}", name, digest);

    match state.storage.get_blob(&digest).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                data.len().to_string().parse().unwrap(),
            );
            headers.insert(
                "Docker-Content-Digest",
                digest.parse().unwrap(),
            );

            Ok((headers, data))
        }
        Ok(None) => Err(RegistryError {
            code: "BLOB_UNKNOWN".to_string(),
            message: format!("Blob {} not found", digest),
            detail: None,
        }),
        Err(e) => {
            error!("Failed to get blob {}: {}", digest, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to retrieve blob".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn head_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    debug!("Head blob: {}/{}", name, digest);

    match state.storage.blob_exists(&digest).await {
        Ok(true) => {
            // For head requests, we need to get the blob to return its size
            match state.storage.get_blob(&digest).await {
                Ok(Some(data)) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        header::CONTENT_TYPE,
                        "application/octet-stream".parse().unwrap(),
                    );
                    headers.insert(
                        header::CONTENT_LENGTH,
                        data.len().to_string().parse().unwrap(),
                    );
                    headers.insert(
                        "Docker-Content-Digest",
                        digest.parse().unwrap(),
                    );

                    Ok((StatusCode::OK, headers))
                }
                Ok(None) => Err(RegistryError {
                    code: "BLOB_UNKNOWN".to_string(),
                    message: format!("Blob {} not found", digest),
                    detail: None,
                }),
                Err(e) => {
                    error!("Failed to get blob size {}: {}", digest, e);
                    Err(RegistryError {
                        code: "UNKNOWN".to_string(),
                        message: "Failed to check blob".to_string(),
                        detail: None,
                    })
                }
            }
        }
        Ok(false) => Err(RegistryError {
            code: "BLOB_UNKNOWN".to_string(),
            message: format!("Blob {} not found", digest),
            detail: None,
        }),
        Err(e) => {
            error!("Failed to check blob {}: {}", digest, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to check blob".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn delete_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Deleting blob: {}/{}", name, digest);

    match state.storage.delete_blob(&digest).await {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            error!("Failed to delete blob {}: {}", digest, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to delete blob".to_string(),
                detail: None,
            })
        }
    }
}