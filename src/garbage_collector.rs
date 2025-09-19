use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::config::{Config, GarbageCollectorConfig};
use crate::storage::{BlobMetadata, ManifestMetadata, StorageBackend};
use std::sync::Arc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GarbageCollectorMetrics {
    pub orphaned_blobs_found: usize,
    pub orphaned_manifests_found: usize,
    pub blobs_deleted: usize,
    pub manifests_deleted: usize,
    pub bytes_freed: u64,
    pub run_duration_seconds: f64,
}

pub struct GarbageCollector {
    config: GarbageCollectorConfig,
    storage: Arc<dyn StorageBackend>,
}

impl GarbageCollector {
    pub fn new(config: GarbageCollectorConfig, storage: Arc<dyn StorageBackend>) -> Self {
        Self { config, storage }
    }

    /// Start the garbage collector background task
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Garbage collector is disabled");
            return Ok(());
        }

        info!(
            "Starting garbage collector with {} hour interval",
            self.config.interval_hours
        );

        let mut interval = interval(std::time::Duration::from_secs(
            self.config.interval_hours * 3600,
        ));

        loop {
            interval.tick().await;

            if let Err(e) = self.run_garbage_collection().await {
                error!("Garbage collection failed: {}", e);
            }
        }
    }

    /// Run a single garbage collection cycle
    pub async fn run_garbage_collection(&self) -> Result<GarbageCollectorMetrics> {
        let start_time = std::time::Instant::now();
        info!("Starting garbage collection run");

        let mut metrics = GarbageCollectorMetrics {
            orphaned_blobs_found: 0,
            orphaned_manifests_found: 0,
            blobs_deleted: 0,
            manifests_deleted: 0,
            bytes_freed: 0,
            run_duration_seconds: 0.0,
        };

        // Step 1: Find all referenced blobs from manifests
        let referenced_blobs = self.find_referenced_blobs().await?;
        info!("Found {} referenced blobs", referenced_blobs.len());

        // Step 2: Find all existing blobs
        let all_blobs = self.find_all_blobs().await?;
        info!("Found {} total blobs in storage", all_blobs.len());

        // Step 3: Identify orphaned blobs
        let orphaned_blobs = self.find_orphaned_blobs(&all_blobs, &referenced_blobs).await?;
        metrics.orphaned_blobs_found = orphaned_blobs.len();
        info!("Found {} orphaned blobs", orphaned_blobs.len());

        // Step 4: Delete orphaned blobs (respecting grace period)
        let (deleted_blobs, bytes_freed) = self.delete_orphaned_blobs(&orphaned_blobs).await?;
        metrics.blobs_deleted = deleted_blobs;
        metrics.bytes_freed = bytes_freed;

        // Step 5: Find and clean orphaned manifests
        let orphaned_manifests = self.find_orphaned_manifests().await?;
        metrics.orphaned_manifests_found = orphaned_manifests.len();

        if !orphaned_manifests.is_empty() {
            info!("Found {} orphaned manifests", orphaned_manifests.len());
            metrics.manifests_deleted = self.delete_orphaned_manifests(&orphaned_manifests).await?;
        }

        metrics.run_duration_seconds = start_time.elapsed().as_secs_f64();

        info!(
            "Garbage collection completed: {} blobs deleted, {} manifests deleted, {} bytes freed, took {:.2}s",
            metrics.blobs_deleted,
            metrics.manifests_deleted,
            metrics.bytes_freed,
            metrics.run_duration_seconds
        );

        Ok(metrics)
    }

    /// Find all blobs referenced by manifests
    async fn find_referenced_blobs(&self) -> Result<HashSet<String>> {
        let mut referenced_blobs = HashSet::new();

        // Get all repositories
        let repositories = self.storage.list_repositories().await?;

        for repository in repositories {
            // Get all tags for this repository
            let tags = self.storage.list_tags(&repository).await?;

            for tag in tags {
                // Get manifest for each tag
                if let Ok(Some(manifest_data)) = self.storage.get_manifest(&repository, &tag).await {
                    if let Ok(manifest) = serde_json::from_slice::<serde_json::Value>(&manifest_data) {
                        // Extract blob references from manifest
                        self.extract_blob_references(&manifest, &mut referenced_blobs);
                    }
                }
            }

            // Also check manifest lists and other manifest types
            if let Ok(manifests) = self.storage.list_manifests(&repository).await {
                for manifest_digest in manifests {
                    if let Ok(manifest_data) = self.storage.get_manifest_by_digest(&repository, &manifest_digest).await {
                        if let Ok(manifest) = serde_json::from_slice::<serde_json::Value>(&manifest_data) {
                            self.extract_blob_references(&manifest, &mut referenced_blobs);
                        }
                    }
                }
            }
        }

        Ok(referenced_blobs)
    }

    /// Extract blob references from a manifest JSON
    fn extract_blob_references(&self, manifest: &serde_json::Value, referenced_blobs: &mut HashSet<String>) {
        // Extract config blob if present
        if let Some(config) = manifest.get("config") {
            if let Some(digest) = config.get("digest").and_then(|d| d.as_str()) {
                referenced_blobs.insert(digest.to_string());
            }
        }

        // Extract layer blobs
        if let Some(layers) = manifest.get("layers").and_then(|l| l.as_array()) {
            for layer in layers {
                if let Some(digest) = layer.get("digest").and_then(|d| d.as_str()) {
                    referenced_blobs.insert(digest.to_string());
                }
            }
        }

        // Handle manifest lists (index manifests)
        if let Some(manifests) = manifest.get("manifests").and_then(|m| m.as_array()) {
            for sub_manifest in manifests {
                if let Some(digest) = sub_manifest.get("digest").and_then(|d| d.as_str()) {
                    referenced_blobs.insert(digest.to_string());
                }
            }
        }

        // Handle foreign layers (though these shouldn't be deleted anyway)
        if let Some(layers) = manifest.get("foreignLayers").and_then(|l| l.as_array()) {
            for layer in layers {
                if let Some(digest) = layer.get("digest").and_then(|d| d.as_str()) {
                    referenced_blobs.insert(digest.to_string());
                }
            }
        }
    }

    /// Find all blobs in storage
    async fn find_all_blobs(&self) -> Result<Vec<String>> {
        self.storage.list_all_blobs().await
    }

    /// Find orphaned blobs by comparing all blobs with referenced blobs
    async fn find_orphaned_blobs(
        &self,
        all_blobs: &[String],
        referenced_blobs: &HashSet<String>,
    ) -> Result<Vec<String>> {
        let mut orphaned = Vec::new();

        for blob_digest in all_blobs {
            if !referenced_blobs.contains(blob_digest) {
                // Check if blob is old enough to be considered for deletion
                if let Ok(metadata) = self.storage.get_blob_metadata(blob_digest).await {
                    let grace_period = Duration::hours(self.config.grace_period_hours as i64);
                    let cutoff_time = Utc::now() - grace_period;

                    if metadata.created_at < cutoff_time {
                        orphaned.push(blob_digest.clone());
                    }
                }
            }
        }

        Ok(orphaned)
    }

    /// Delete orphaned blobs
    async fn delete_orphaned_blobs(&self, orphaned_blobs: &[String]) -> Result<(usize, u64)> {
        let mut deleted_count = 0;
        let mut bytes_freed = 0u64;

        let blobs_to_process = if orphaned_blobs.len() > self.config.max_blobs_per_run {
            warn!(
                "Limiting deletion to {} blobs per run (found {} orphaned)",
                self.config.max_blobs_per_run,
                orphaned_blobs.len()
            );
            &orphaned_blobs[..self.config.max_blobs_per_run]
        } else {
            orphaned_blobs
        };

        for blob_digest in blobs_to_process {
            if self.config.dry_run {
                info!("DRY RUN: Would delete blob {}", blob_digest);
                deleted_count += 1;
                continue;
            }

            // Get blob size before deletion
            if let Ok(metadata) = self.storage.get_blob_metadata(blob_digest).await {
                bytes_freed += metadata.size;
            }

            match self.storage.delete_blob(blob_digest).await {
                Ok(_) => {
                    info!("Deleted orphaned blob {}", blob_digest);
                    deleted_count += 1;
                }
                Err(e) => {
                    error!("Failed to delete blob {}: {}", blob_digest, e);
                }
            }
        }

        Ok((deleted_count, bytes_freed))
    }

    /// Find orphaned manifests (manifests not referenced by any tags)
    async fn find_orphaned_manifests(&self) -> Result<Vec<String>> {
        let mut orphaned_manifests = Vec::new();
        let repositories = self.storage.list_repositories().await?;

        for repository in repositories {
            // Get all manifests
            let all_manifests = self.storage.list_manifests(&repository).await?;

            // Get manifests referenced by tags
            let tags = self.storage.list_tags(&repository).await?;
            let mut referenced_manifests = HashSet::new();

            for tag in tags {
                if let Ok(manifest_digest) = self.storage.get_manifest_digest(&repository, &tag).await {
                    referenced_manifests.insert(manifest_digest);
                }
            }

            // Find orphaned manifests
            for manifest_digest in all_manifests {
                if !referenced_manifests.contains(&manifest_digest) {
                    // Check grace period for manifests too
                    if let Ok(metadata) = self.storage.get_manifest_metadata(&repository, &manifest_digest).await {
                        let grace_period = Duration::hours(self.config.grace_period_hours as i64);
                        let cutoff_time = Utc::now() - grace_period;

                        if metadata.created_at < cutoff_time {
                            orphaned_manifests.push(format!("{}:{}", repository, manifest_digest));
                        }
                    }
                }
            }
        }

        Ok(orphaned_manifests)
    }

    /// Delete orphaned manifests
    async fn delete_orphaned_manifests(&self, orphaned_manifests: &[String]) -> Result<usize> {
        let mut deleted_count = 0;

        for manifest_ref in orphaned_manifests {
            let parts: Vec<&str> = manifest_ref.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let (repository, manifest_digest) = (parts[0], parts[1]);

            if self.config.dry_run {
                info!("DRY RUN: Would delete manifest {}:{}", repository, manifest_digest);
                deleted_count += 1;
                continue;
            }

            match self.storage.delete_manifest(repository, manifest_digest).await {
                Ok(_) => {
                    info!("Deleted orphaned manifest {}:{}", repository, manifest_digest);
                    deleted_count += 1;
                }
                Err(e) => {
                    error!("Failed to delete manifest {}:{}: {}", repository, manifest_digest, e);
                }
            }
        }

        Ok(deleted_count)
    }

    /// Manually trigger garbage collection (useful for admin endpoints)
    pub async fn trigger_manual_run(&self) -> Result<GarbageCollectorMetrics> {
        info!("Manual garbage collection triggered");
        self.run_garbage_collection().await
    }
}

// Note: BlobMetadata and ManifestMetadata are now defined in storage::mod