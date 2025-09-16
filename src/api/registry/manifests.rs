use super::RegistryError;
use crate::server::AppState;
use axum::{
    extract::{Path, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    body::Body,
};
use bytes::Bytes;
use sha2::{Digest, Sha256};
use tracing::{debug, error, info};

pub async fn get_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Getting manifest: {}/{}", name, reference);

    match state.storage.get_manifest(&name, &reference).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();

            // Calculate content digest
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let digest = format!("sha256:{:x}", hasher.finalize());

            headers.insert(
                header::CONTENT_TYPE,
                "application/vnd.docker.distribution.manifest.v2+json".parse().unwrap(),
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
            code: "MANIFEST_UNKNOWN".to_string(),
            message: format!("Manifest {}:{} not found", name, reference),
            detail: None,
        }),
        Err(e) => {
            error!("Failed to get manifest {}:{}: {}", name, reference, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to retrieve manifest".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn put_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Putting manifest: {}/{} ({} bytes)", name, reference, body.len());

    // Validate content type
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !content_type.contains("application/vnd.docker.distribution.manifest")
        && !content_type.contains("application/vnd.oci.image.manifest") {
        return Err(RegistryError {
            code: "UNSUPPORTED".to_string(),
            message: "Unsupported manifest media type".to_string(),
            detail: None,
        });
    }

    // Calculate digest
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let digest = format!("sha256:{:x}", hasher.finalize());

    // Store manifest
    match state.storage.put_manifest(&name, &reference, body).await {
        Ok(()) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::LOCATION,
                format!("/v2/{}/manifests/{}", name, reference).parse().unwrap(),
            );
            response_headers.insert(
                "Docker-Content-Digest",
                digest.parse().unwrap(),
            );

            Ok((StatusCode::CREATED, response_headers))
        }
        Err(e) => {
            error!("Failed to store manifest {}:{}: {}", name, reference, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to store manifest".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn head_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    debug!("Head manifest: {}/{}", name, reference);

    match state.storage.get_manifest(&name, &reference).await {
        Ok(Some(data)) => {
            let mut headers = HeaderMap::new();

            // Calculate content digest
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let digest = format!("sha256:{:x}", hasher.finalize());

            headers.insert(
                header::CONTENT_TYPE,
                "application/vnd.docker.distribution.manifest.v2+json".parse().unwrap(),
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
            code: "MANIFEST_UNKNOWN".to_string(),
            message: format!("Manifest {}:{} not found", name, reference),
            detail: None,
        }),
        Err(e) => {
            error!("Failed to check manifest {}:{}: {}", name, reference, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to check manifest".to_string(),
                detail: None,
            })
        }
    }
}

pub async fn delete_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<impl IntoResponse, RegistryError> {
    info!("Deleting manifest: {}/{}", name, reference);

    match state.storage.delete_manifest(&name, &reference).await {
        Ok(()) => Ok(StatusCode::ACCEPTED),
        Err(e) => {
            error!("Failed to delete manifest {}:{}: {}", name, reference, e);
            Err(RegistryError {
                code: "UNKNOWN".to_string(),
                message: "Failed to delete manifest".to_string(),
                detail: None,
            })
        }
    }
}