use crate::config::{StorageConfig, StorageType};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

pub mod filesystem;
pub mod s3;

#[cfg(feature = "ghostbay-storage")]
pub mod ghostbay;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn put_blob(&self, digest: &str, data: Bytes) -> Result<()>;
    async fn get_blob(&self, digest: &str) -> Result<Option<Bytes>>;
    async fn delete_blob(&self, digest: &str) -> Result<()>;
    async fn blob_exists(&self, digest: &str) -> Result<bool>;

    async fn put_manifest(&self, repo: &str, reference: &str, data: Bytes) -> Result<()>;
    async fn get_manifest(&self, repo: &str, reference: &str) -> Result<Option<Bytes>>;
    async fn delete_manifest(&self, repo: &str, reference: &str) -> Result<()>;

    async fn list_repositories(&self) -> Result<Vec<String>>;
    async fn list_tags(&self, repo: &str) -> Result<Vec<String>>;

    async fn get_upload_url(&self, uuid: &str) -> Result<Option<String>>;
    async fn put_upload_chunk(&self, uuid: &str, range: (u64, u64), data: Bytes) -> Result<()>;
    async fn complete_upload(&self, uuid: &str, digest: &str) -> Result<()>;
    async fn cancel_upload(&self, uuid: &str) -> Result<()>;
}

pub async fn create_storage_backend(config: &StorageConfig) -> Result<Arc<dyn StorageBackend>> {
    match config.storage_type {
        StorageType::Filesystem => {
            let path = config.path.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Filesystem storage requires path"))?;
            Ok(Arc::new(filesystem::FilesystemStorage::new(path).await?))
        }
        StorageType::S3 => {
            let s3_config = config.s3.as_ref()
                .ok_or_else(|| anyhow::anyhow!("S3 storage requires s3 config"))?;
            Ok(Arc::new(s3::S3Storage::new(s3_config).await?))
        }
        StorageType::GhostBay => {
            #[cfg(feature = "ghostbay-storage")]
            {
                let ghostbay_config = config.ghostbay.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GhostBay storage requires ghostbay config"))?;
                Ok(Arc::new(ghostbay::GhostBayStorage::new(ghostbay_config).await?))
            }
            #[cfg(not(feature = "ghostbay-storage"))]
            {
                Err(anyhow::anyhow!("GhostBay storage not available - enable ghostbay-storage feature"))
            }
        }
    }
}