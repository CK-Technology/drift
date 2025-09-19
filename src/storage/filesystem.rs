use super::{BlobMetadata, ManifestMetadata, StorageBackend};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error};

pub struct FilesystemStorage {
    base_path: PathBuf,
}

impl FilesystemStorage {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create necessary directories
        fs::create_dir_all(&base_path).await?;
        fs::create_dir_all(base_path.join("blobs")).await?;
        fs::create_dir_all(base_path.join("manifests")).await?;
        fs::create_dir_all(base_path.join("uploads")).await?;

        debug!("Initialized filesystem storage at: {:?}", base_path);

        Ok(Self { base_path })
    }

    fn blob_path(&self, digest: &str) -> PathBuf {
        // Store blobs in subdirectories based on first 2 chars of digest for performance
        let prefix = &digest[0..2];
        self.base_path.join("blobs").join(prefix).join(digest)
    }

    fn manifest_path(&self, repo: &str, reference: &str) -> PathBuf {
        self.base_path
            .join("manifests")
            .join(repo)
            .join(reference)
    }

    fn upload_path(&self, uuid: &str) -> PathBuf {
        self.base_path.join("uploads").join(uuid)
    }
}

#[async_trait]
impl StorageBackend for FilesystemStorage {
    async fn put_blob(&self, digest: &str, data: Bytes) -> Result<()> {
        let path = self.blob_path(digest);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&path, &data).await?;
        debug!("Stored blob {} ({} bytes)", digest, data.len());
        Ok(())
    }

    async fn get_blob(&self, digest: &str) -> Result<Option<Bytes>> {
        let path = self.blob_path(digest);

        match fs::read(&path).await {
            Ok(data) => {
                debug!("Retrieved blob {} ({} bytes)", digest, data.len());
                Ok(Some(data.into()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => {
                error!("Failed to read blob {}: {}", digest, e);
                Err(e.into())
            }
        }
    }

    async fn delete_blob(&self, digest: &str) -> Result<()> {
        let path = self.blob_path(digest);

        match fs::remove_file(&path).await {
            Ok(()) => {
                debug!("Deleted blob {}", digest);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => {
                error!("Failed to delete blob {}: {}", digest, e);
                Err(e.into())
            }
        }
    }

    async fn blob_exists(&self, digest: &str) -> Result<bool> {
        let path = self.blob_path(digest);
        Ok(path.exists())
    }

    async fn put_manifest(&self, repo: &str, reference: &str, data: Bytes) -> Result<()> {
        let path = self.manifest_path(repo, reference);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&path, &data).await?;
        debug!("Stored manifest {}/{} ({} bytes)", repo, reference, data.len());
        Ok(())
    }

    async fn get_manifest(&self, repo: &str, reference: &str) -> Result<Option<Bytes>> {
        let path = self.manifest_path(repo, reference);

        match fs::read(&path).await {
            Ok(data) => {
                debug!("Retrieved manifest {}/{} ({} bytes)", repo, reference, data.len());
                Ok(Some(data.into()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => {
                error!("Failed to read manifest {}/{}: {}", repo, reference, e);
                Err(e.into())
            }
        }
    }

    async fn delete_manifest(&self, repo: &str, reference: &str) -> Result<()> {
        let path = self.manifest_path(repo, reference);

        match fs::remove_file(&path).await {
            Ok(()) => {
                debug!("Deleted manifest {}/{}", repo, reference);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => {
                error!("Failed to delete manifest {}/{}: {}", repo, reference, e);
                Err(e.into())
            }
        }
    }

    async fn list_repositories(&self) -> Result<Vec<String>> {
        let manifests_path = self.base_path.join("manifests");
        let mut repos = Vec::new();

        if !manifests_path.exists() {
            return Ok(repos);
        }

        let mut entries = fs::read_dir(&manifests_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    repos.push(name.to_string());
                }
            }
        }

        repos.sort();
        Ok(repos)
    }

    async fn list_tags(&self, repo: &str) -> Result<Vec<String>> {
        let repo_path = self.base_path.join("manifests").join(repo);
        let mut tags = Vec::new();

        if !repo_path.exists() {
            return Ok(tags);
        }

        let mut entries = fs::read_dir(&repo_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    tags.push(name.to_string());
                }
            }
        }

        tags.sort();
        Ok(tags)
    }

    async fn get_upload_url(&self, uuid: &str) -> Result<Option<String>> {
        let path = self.upload_path(uuid);
        if path.exists() {
            Ok(Some(format!("/v2/uploads/{}", uuid)))
        } else {
            Ok(None)
        }
    }

    async fn put_upload_chunk(&self, uuid: &str, range: (u64, u64), data: Bytes) -> Result<()> {
        let path = self.upload_path(uuid);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // For filesystem storage, we'll append chunks in order
        // In a real implementation, you'd want to handle random writes properly
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .await?;

        use tokio::io::{AsyncSeekExt, AsyncWriteExt};
        file.seek(std::io::SeekFrom::Start(range.0)).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        debug!("Wrote upload chunk {} range {:?} ({} bytes)", uuid, range, data.len());
        Ok(())
    }

    async fn complete_upload(&self, uuid: &str, digest: &str) -> Result<()> {
        let upload_path = self.upload_path(uuid);
        let blob_path = self.blob_path(digest);

        if let Some(parent) = blob_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Move upload to blob storage
        fs::rename(&upload_path, &blob_path).await?;
        debug!("Completed upload {} -> blob {}", uuid, digest);
        Ok(())
    }

    async fn cancel_upload(&self, uuid: &str) -> Result<()> {
        let path = self.upload_path(uuid);

        match fs::remove_file(&path).await {
            Ok(()) => {
                debug!("Cancelled upload {}", uuid);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => {
                error!("Failed to cancel upload {}: {}", uuid, e);
                Err(e.into())
            }
        }
    }

    // Garbage collection methods
    async fn list_all_blobs(&self) -> Result<Vec<String>> {
        let mut blobs = Vec::new();
        let blobs_path = self.base_path.join("blobs");

        if !blobs_path.exists() {
            return Ok(blobs);
        }

        // Walk through all subdirectories in blobs/
        let mut prefix_entries = fs::read_dir(&blobs_path).await?;

        while let Some(prefix_entry) = prefix_entries.next_entry().await? {
            if prefix_entry.file_type().await?.is_dir() {
                let prefix_path = prefix_entry.path();
                let mut blob_entries = fs::read_dir(&prefix_path).await?;

                while let Some(blob_entry) = blob_entries.next_entry().await? {
                    if blob_entry.file_type().await?.is_file() {
                        if let Some(digest) = blob_entry.file_name().to_str() {
                            blobs.push(digest.to_string());
                        }
                    }
                }
            }
        }

        Ok(blobs)
    }

    async fn list_manifests(&self, repo: &str) -> Result<Vec<String>> {
        let mut manifests = Vec::new();
        let repo_path = self.base_path.join("manifests").join(repo);

        if !repo_path.exists() {
            return Ok(manifests);
        }

        let mut entries = fs::read_dir(&repo_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    // For manifest digests, we need to compute the SHA256 of the file
                    let manifest_data = fs::read(entry.path()).await?;
                    let digest = format!("sha256:{:x}", Sha256::digest(&manifest_data));
                    manifests.push(digest);
                }
            }
        }

        Ok(manifests)
    }

    async fn get_blob_metadata(&self, digest: &str) -> Result<BlobMetadata> {
        let path = self.blob_path(digest);
        let metadata = fs::metadata(&path).await?;

        let created_at = metadata.created()
            .or_else(|_| metadata.modified())?
            .into();

        Ok(BlobMetadata {
            size: metadata.len(),
            created_at,
        })
    }

    async fn get_manifest_metadata(&self, repo: &str, digest: &str) -> Result<ManifestMetadata> {
        // For digest-based lookups, we need to find the manifest file
        let repo_path = self.base_path.join("manifests").join(repo);

        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found: {}", repo));
        }

        let mut entries = fs::read_dir(&repo_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                // Check if this file's digest matches
                let manifest_data = fs::read(entry.path()).await?;
                let file_digest = format!("sha256:{:x}", Sha256::digest(&manifest_data));

                if file_digest == digest {
                    let metadata = fs::metadata(entry.path()).await?;
                    let created_at = metadata.created()
                        .or_else(|_| metadata.modified())?
                        .into();

                    return Ok(ManifestMetadata {
                        size: metadata.len(),
                        created_at,
                    });
                }
            }
        }

        Err(anyhow::anyhow!("Manifest not found: {}", digest))
    }

    async fn get_manifest_by_digest(&self, repo: &str, digest: &str) -> Result<Bytes> {
        let repo_path = self.base_path.join("manifests").join(repo);

        if !repo_path.exists() {
            return Err(anyhow::anyhow!("Repository not found: {}", repo));
        }

        let mut entries = fs::read_dir(&repo_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                // Check if this file's digest matches
                let manifest_data = fs::read(entry.path()).await?;
                let file_digest = format!("sha256:{:x}", Sha256::digest(&manifest_data));

                if file_digest == digest {
                    return Ok(manifest_data.into());
                }
            }
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