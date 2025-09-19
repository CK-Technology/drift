use super::{BlobMetadata, ManifestMetadata, StorageBackend};
use crate::config::S3Config;
use anyhow::Result;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::{config::Credentials, Client, Config};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, error, info};

pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub async fn new(config: &S3Config) -> Result<Self> {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "drift-s3",
        );

        let mut s3_config_builder = Config::builder()
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials);

        // Configure for MinIO/custom S3 endpoints
        if config.path_style {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        if !config.endpoint.starts_with("https://s3.") {
            // Custom endpoint (like MinIO)
            s3_config_builder = s3_config_builder.endpoint_url(&config.endpoint);
        }

        let s3_config = s3_config_builder.build();
        let client = Client::from_conf(s3_config);

        // Test connection
        match client.head_bucket().bucket(&config.bucket).send().await {
            Ok(_) => info!("✅ Connected to S3 bucket: {}", config.bucket),
            Err(e) => {
                error!("❌ Failed to connect to S3 bucket {}: {}", config.bucket, e);
                return Err(anyhow::anyhow!("S3 connection failed: {}", e));
            }
        }

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
        })
    }

    fn blob_key(&self, digest: &str) -> String {
        format!("blobs/{}/{}", &digest[0..2], digest)
    }

    fn manifest_key(&self, repo: &str, reference: &str) -> String {
        format!("manifests/{}/{}", repo, reference)
    }

    fn upload_key(&self, uuid: &str) -> String {
        format!("uploads/{}", uuid)
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn put_blob(&self, digest: &str, data: Bytes) -> Result<()> {
        let key = self.blob_key(digest);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data.clone()))
            .content_type("application/octet-stream")
            .metadata("digest", digest)
            .send()
            .await?;

        debug!("Stored blob {} in S3 ({} bytes)", digest, data.len());
        Ok(())
    }

    async fn get_blob(&self, digest: &str) -> Result<Option<Bytes>> {
        let key = self.blob_key(digest);

        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(resp) => {
                let data = resp.body.collect().await?.into_bytes();
                debug!("Retrieved blob {} from S3 ({} bytes)", digest, data.len());
                Ok(Some(data))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    error!("Failed to get blob {} from S3: {}", digest, e);
                    Err(e.into())
                }
            }
        }
    }

    async fn delete_blob(&self, digest: &str) -> Result<()> {
        let key = self.blob_key(digest);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        debug!("Deleted blob {} from S3", digest);
        Ok(())
    }

    async fn blob_exists(&self, digest: &str) -> Result<bool> {
        let key = self.blob_key(digest);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    async fn put_manifest(&self, repo: &str, reference: &str, data: Bytes) -> Result<()> {
        let key = self.manifest_key(repo, reference);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data.clone()))
            .content_type("application/vnd.docker.distribution.manifest.v2+json")
            .metadata("repository", repo)
            .metadata("reference", reference)
            .send()
            .await?;

        debug!("Stored manifest {}/{} in S3 ({} bytes)", repo, reference, data.len());
        Ok(())
    }

    async fn get_manifest(&self, repo: &str, reference: &str) -> Result<Option<Bytes>> {
        let key = self.manifest_key(repo, reference);

        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(resp) => {
                let data = resp.body.collect().await?.into_bytes();
                debug!("Retrieved manifest {}/{} from S3 ({} bytes)", repo, reference, data.len());
                Ok(Some(data))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    error!("Failed to get manifest {}/{} from S3: {}", repo, reference, e);
                    Err(e.into())
                }
            }
        }
    }

    async fn delete_manifest(&self, repo: &str, reference: &str) -> Result<()> {
        let key = self.manifest_key(repo, reference);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        debug!("Deleted manifest {}/{} from S3", repo, reference);
        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        let mut repos = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix("manifests/")
                .delimiter("/");

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let resp = request.send().await?;

            // Extract repository names from common prefixes
            if let Some(prefixes) = resp.common_prefixes {
                for prefix in prefixes {
                    if let Some(prefix_str) = prefix.prefix {
                        if let Some(repo_name) = prefix_str.strip_prefix("manifests/").and_then(|s| s.strip_suffix("/")) {
                            repos.push(repo_name.to_string());
                        }
                    }
                }
            }

            if resp.is_truncated == Some(true) {
                continuation_token = resp.next_continuation_token;
            } else {
                break;
            }
        }

        repos.sort();
        repos.dedup();
        Ok(repos)
    }

    async fn list_tags(&self, repo: &str) -> Result<Vec<String>> {
        let mut tags = Vec::new();
        let prefix = format!("manifests/{}/", repo);
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let resp = request.send().await?;

            if let Some(objects) = resp.contents {
                for object in objects {
                    if let Some(key) = object.key {
                        if let Some(tag) = key.strip_prefix(&prefix) {
                            tags.push(tag.to_string());
                        }
                    }
                }
            }

            if resp.is_truncated == Some(true) {
                continuation_token = resp.next_continuation_token;
            } else {
                break;
            }
        }

        tags.sort();
        Ok(tags)
    }

    async fn get_upload_url(&self, uuid: &str) -> Result<Option<String>> {
        // For S3, we track uploads using metadata or a separate key
        let key = format!("uploads/{}/metadata", uuid);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(Some(format!("/v2/uploads/{}", uuid))),
            Err(_) => Ok(None),
        }
    }

    async fn put_upload_chunk(&self, uuid: &str, range: (u64, u64), data: Bytes) -> Result<()> {
        // For simplicity, store chunks as separate objects
        // In production, you'd use S3 multipart uploads
        let key = format!("uploads/{}/chunk-{}-{}", uuid, range.0, range.1);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data.clone()))
            .metadata("range_start", range.0.to_string())
            .metadata("range_end", range.1.to_string())
            .send()
            .await?;

        debug!("Stored upload chunk {} range {:?} in S3", uuid, range);
        Ok(())
    }

    async fn complete_upload(&self, uuid: &str, digest: &str) -> Result<()> {
        // Collect all chunks and combine them into the final blob
        let prefix = format!("uploads/{}/chunk-", uuid);
        let mut chunks = Vec::new();

        let resp = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await?;

        if let Some(objects) = resp.contents {
            for object in objects {
                if let Some(key) = object.key {
                    chunks.push(key);
                }
            }
        }

        // Sort chunks by range
        chunks.sort();

        // Combine chunks into final blob
        let mut combined_data = Vec::new();
        for chunk_key in &chunks {
            let chunk_resp = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(chunk_key)
                .send()
                .await?;

            let chunk_data = chunk_resp.body.collect().await?.into_bytes();
            combined_data.extend_from_slice(&chunk_data);
        }

        // Store as final blob
        self.put_blob(digest, combined_data.into()).await?;

        // Clean up upload chunks
        for chunk_key in chunks {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(&chunk_key)
                .send()
                .await?;
        }

        debug!("Completed upload {} -> blob {}", uuid, digest);
        Ok(())
    }

    async fn cancel_upload(&self, uuid: &str) -> Result<()> {
        // Delete all upload-related objects
        let prefix = format!("uploads/{}/", uuid);

        let resp = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await?;

        if let Some(objects) = resp.contents {
            for object in objects {
                if let Some(key) = object.key {
                    self.client
                        .delete_object()
                        .bucket(&self.bucket)
                        .key(&key)
                        .send()
                        .await?;
                }
            }
        }

        debug!("Cancelled upload {}", uuid);
        Ok(())
    }

    // Garbage collection methods
    async fn list_all_blobs(&self) -> Result<Vec<String>> {
        let mut blobs = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix("blobs/");

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Extract digest from key (remove "blobs/" prefix)
                        if let Some(digest) = key.strip_prefix("blobs/") {
                            blobs.push(digest.to_string());
                        }
                    }
                }
            }

            if !response.is_truncated.unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token;
        }

        Ok(blobs)
    }

    async fn list_manifests(&self, repo: &str) -> Result<Vec<String>> {
        let mut manifests = Vec::new();
        let prefix = format!("manifests/{}/", repo);
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // For manifest digests, we need to get the object and compute its hash
                        match self.client
                            .get_object()
                            .bucket(&self.bucket)
                            .key(&key)
                            .send()
                            .await {
                            Ok(response) => {
                                let body = response.body.collect().await?;
                                let digest = format!("sha256:{:x}", Sha256::digest(&body.into_bytes()));
                                manifests.push(digest);
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }

            if !response.is_truncated.unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token;
        }

        Ok(manifests)
    }

    async fn get_blob_metadata(&self, digest: &str) -> Result<BlobMetadata> {
        let key = self.blob_key(digest);

        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;

        let size = response.content_length.unwrap_or(0) as u64;
        let created_at = response.last_modified
            .map(|dt| chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now))
            .unwrap_or_else(Utc::now);

        Ok(BlobMetadata { size, created_at })
    }

    async fn get_manifest_metadata(&self, repo: &str, digest: &str) -> Result<ManifestMetadata> {
        // For digest-based lookups, we need to find the manifest file
        let prefix = format!("manifests/{}/", repo);
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Check if this file's digest matches
                        if let Ok(obj_response) = self.client
                            .get_object()
                            .bucket(&self.bucket)
                            .key(&key)
                            .send()
                            .await {
                            let body = obj_response.body.collect().await?;
                            let file_digest = format!("sha256:{:x}", Sha256::digest(&body.into_bytes()));

                            if file_digest == digest {
                                let head_response = self
                                    .client
                                    .head_object()
                                    .bucket(&self.bucket)
                                    .key(&key)
                                    .send()
                                    .await?;

                                let size = head_response.content_length.unwrap_or(0) as u64;
                                let created_at = head_response.last_modified
                                    .map(|dt| chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now))
                                    .unwrap_or_else(Utc::now);

                                return Ok(ManifestMetadata { size, created_at });
                            }
                        }
                    }
                }
            }

            if !response.is_truncated.unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token;
        }

        Err(anyhow::anyhow!("Manifest not found: {}", digest))
    }

    async fn get_manifest_by_digest(&self, repo: &str, digest: &str) -> Result<Bytes> {
        let prefix = format!("manifests/{}/", repo);
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Check if this file's digest matches
                        if let Ok(obj_response) = self.client
                            .get_object()
                            .bucket(&self.bucket)
                            .key(&key)
                            .send()
                            .await {
                            let body = obj_response.body.collect().await?;
                            let manifest_data = body.into_bytes();
                            let file_digest = format!("sha256:{:x}", Sha256::digest(&manifest_data));

                            if file_digest == digest {
                                return Ok(manifest_data.into());
                            }
                        }
                    }
                }
            }

            if !response.is_truncated.unwrap_or(false) {
                break;
            }

            continuation_token = response.next_continuation_token;
        }

        Err(anyhow::anyhow!("Manifest not found: {}", digest))
    }

    async fn get_manifest_digest(&self, repo: &str, reference: &str) -> Result<String> {
        let manifest_data = self.get_manifest(repo, reference).await?
            .ok_or_else(|| anyhow::anyhow!("Manifest not found: {}/{}", repo, reference))?;

        let digest = format!("sha256:{:x}", Sha256::digest(&manifest_data));
        Ok(digest)
    }
}