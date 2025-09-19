use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::OptimizationConfig;
use crate::storage::StorageBackend;

/// Automated image optimization service for drift registry
/// Performs layer deduplication, compression optimization, and vulnerability scanning
#[derive(Clone)]
pub struct OptimizationService {
    config: OptimizationConfig,
    storage: Arc<dyn StorageBackend>,
    optimization_cache: Arc<RwLock<HashMap<String, OptimizationResult>>>,
    layer_index: Arc<RwLock<LayerIndex>>,
}

/// Layer index for tracking duplicate layers across images
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LayerIndex {
    /// Map of layer digest to metadata
    pub layers: HashMap<String, LayerMetadata>,
    /// Map of layer content hash to digest (for deduplication)
    pub content_map: HashMap<String, String>,
    /// Size statistics
    pub total_layers: usize,
    pub total_size_bytes: u64,
    pub deduplicated_size_bytes: u64,
}

/// Metadata about a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub digest: String,
    pub size: u64,
    pub media_type: String,
    pub content_hash: String, // Hash of actual content (not compressed)
    pub compression: CompressionType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub reference_count: usize, // Number of images using this layer
    pub optimization_status: OptimizationStatus,
}

/// Compression types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionType {
    #[serde(rename = "gzip")]
    Gzip,
    #[serde(rename = "zstd")]
    Zstd,
    #[serde(rename = "lz4")]
    Lz4,
    #[serde(rename = "brotli")]
    Brotli,
    #[serde(rename = "uncompressed")]
    Uncompressed,
}

/// Optimization status for layers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "optimizing")]
    Optimizing,
    #[serde(rename = "optimized")]
    Optimized,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "skipped")]
    Skipped,
}

/// Result of optimization process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_digest: String,
    pub optimized_digest: Option<String>,
    pub original_size: u64,
    pub optimized_size: u64,
    pub compression_ratio: f64,
    pub optimization_type: OptimizationType,
    pub processing_time_ms: u64,
    pub status: OptimizationStatus,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Types of optimization performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OptimizationType {
    #[serde(rename = "compression")]
    Compression,
    #[serde(rename = "deduplication")]
    Deduplication,
    #[serde(rename = "layer_squashing")]
    LayerSquashing,
    #[serde(rename = "base_image_optimization")]
    BaseImageOptimization,
}

/// Optimization policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationPolicy {
    pub enable_compression_optimization: bool,
    pub enable_layer_deduplication: bool,
    pub enable_layer_squashing: bool,
    pub enable_base_image_optimization: bool,
    pub preferred_compression: CompressionType,
    pub min_layer_size_bytes: u64, // Don't optimize layers smaller than this
    pub max_optimization_time_seconds: u64,
    pub preserve_original: bool,
    pub optimization_schedule: OptimizationSchedule,
}

/// When to run optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationSchedule {
    #[serde(rename = "immediate")]
    Immediate, // On upload
    #[serde(rename = "background")]
    Background, // Background processing
    #[serde(rename = "scheduled")]
    Scheduled, // Cron-like schedule
}

/// Trait for optimization backends
#[async_trait]
pub trait OptimizationBackend: Send + Sync {
    async fn optimize_layer(
        &self,
        layer_data: &[u8],
        current_compression: CompressionType,
        target_compression: CompressionType,
    ) -> Result<Vec<u8>>;

    async fn analyze_layer(&self, layer_data: &[u8]) -> Result<LayerAnalysis>;
    async fn detect_duplicates(&self, layers: &[LayerMetadata]) -> Result<Vec<DuplicateGroup>>;
}

/// Analysis result for a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAnalysis {
    pub content_type: String,
    pub file_count: usize,
    pub directory_count: usize,
    pub largest_files: Vec<FileInfo>,
    pub compression_potential: f64, // Estimated compression ratio improvement
    pub duplicate_content_ratio: f64, // Ratio of duplicate content within layer
}

/// Information about files in a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub hash: String,
}

/// Group of duplicate layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub canonical_digest: String,
    pub duplicate_digests: Vec<String>,
    pub total_size_saved: u64,
}

impl OptimizationService {
    pub async fn new(
        config: OptimizationConfig,
        storage: Arc<dyn StorageBackend>,
    ) -> Result<Self> {
        info!("Initializing image optimization service");

        let service = Self {
            config,
            storage,
            optimization_cache: Arc::new(RwLock::new(HashMap::new())),
            layer_index: Arc::new(RwLock::new(LayerIndex::default())),
        };

        // Load existing layer index
        service.load_layer_index().await?;

        info!("Image optimization service initialized successfully");
        Ok(service)
    }

    /// Optimize a layer (compression, deduplication, etc.)
    pub async fn optimize_layer(
        &self,
        layer_digest: &str,
        layer_data: &[u8],
        policy: &OptimizationPolicy,
    ) -> Result<OptimizationResult> {
        info!("Optimizing layer: {}", layer_digest);
        let start_time = std::time::Instant::now();

        // Check if already optimized
        if let Some(cached_result) = self.get_optimization_result(layer_digest).await {
            if cached_result.status == OptimizationStatus::Optimized {
                debug!("Layer already optimized, returning cached result");
                return Ok(cached_result);
            }
        }

        // Analyze layer
        let analysis = self.analyze_layer(layer_data).await?;
        info!("Layer analysis: {} files, {} directories, {:.2}% compression potential",
            analysis.file_count, analysis.directory_count, analysis.compression_potential * 100.0);

        // Skip optimization if layer is too small
        if layer_data.len() < policy.min_layer_size_bytes as usize {
            return Ok(OptimizationResult {
                original_digest: layer_digest.to_string(),
                optimized_digest: None,
                original_size: layer_data.len() as u64,
                optimized_size: layer_data.len() as u64,
                compression_ratio: 1.0,
                optimization_type: OptimizationType::Compression,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                status: OptimizationStatus::Skipped,
                error_message: Some("Layer too small for optimization".to_string()),
                created_at: chrono::Utc::now(),
            });
        }

        // Perform optimization based on policy
        let mut optimized_data = layer_data.to_vec();
        let mut optimization_type = OptimizationType::Compression;

        // Compression optimization
        if policy.enable_compression_optimization && analysis.compression_potential > 0.1 {
            match self.optimize_compression(&optimized_data, &policy.preferred_compression).await {
                Ok(compressed_data) => {
                    if compressed_data.len() < optimized_data.len() {
                        info!("Compression optimization: {} -> {} bytes ({:.2}% reduction)",
                            optimized_data.len(), compressed_data.len(),
                            (1.0 - compressed_data.len() as f64 / optimized_data.len() as f64) * 100.0);
                        optimized_data = compressed_data;
                    }
                }
                Err(e) => {
                    warn!("Compression optimization failed: {}", e);
                }
            }
        }

        // Check for deduplication opportunities
        if policy.enable_layer_deduplication {
            if let Some(duplicate_digest) = self.find_duplicate_layer(&optimized_data).await? {
                info!("Found duplicate layer: {} -> {}", layer_digest, duplicate_digest);
                optimization_type = OptimizationType::Deduplication;

                // Update reference count for the canonical layer
                self.increment_layer_references(&duplicate_digest).await?;

                return Ok(OptimizationResult {
                    original_digest: layer_digest.to_string(),
                    optimized_digest: Some(duplicate_digest),
                    original_size: layer_data.len() as u64,
                    optimized_size: 0, // No additional storage needed
                    compression_ratio: 0.0, // Complete deduplication
                    optimization_type,
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    status: OptimizationStatus::Optimized,
                    error_message: None,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        // Store optimized layer if different from original
        let optimized_digest = if optimized_data != layer_data {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&optimized_data);
            let new_digest = format!("sha256:{}", hex::encode(hasher.finalize()));
            let key = format!("blobs/{}", new_digest);
            self.storage.put_blob(&key, optimized_data.clone().into()).await?;
            Some(new_digest)
        } else {
            None
        };

        // Update layer index
        self.update_layer_index(layer_digest, &optimized_data, &analysis).await?;

        let result = OptimizationResult {
            original_digest: layer_digest.to_string(),
            optimized_digest,
            original_size: layer_data.len() as u64,
            optimized_size: optimized_data.len() as u64,
            compression_ratio: optimized_data.len() as f64 / layer_data.len() as f64,
            optimization_type,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            status: OptimizationStatus::Optimized,
            error_message: None,
            created_at: chrono::Utc::now(),
        };

        // Cache result
        self.cache_optimization_result(layer_digest, &result).await;

        info!("Layer optimization completed: {:.2}% size reduction",
            (1.0 - result.compression_ratio) * 100.0);
        Ok(result)
    }

    /// Optimize image manifest (layer deduplication, base image optimization)
    pub async fn optimize_manifest(
        &self,
        manifest_content: &[u8],
        policy: &OptimizationPolicy,
    ) -> Result<Vec<u8>> {
        debug!("Optimizing image manifest");

        // Parse manifest
        let mut manifest: serde_json::Value = serde_json::from_slice(manifest_content)?;

        // Extract layers
        if let Some(layers) = manifest.get_mut("layers").and_then(|l| l.as_array_mut()) {
            let mut optimized = false;

            for layer in layers.iter_mut() {
                if let Some(digest) = layer.get("digest").and_then(|d| d.as_str()) {
                    let digest_str = digest.to_string();
                    // Check if we have an optimized version of this layer
                    if let Some(result) = self.get_optimization_result(&digest_str).await {
                        if let Some(optimized_digest) = result.optimized_digest {
                            // Update manifest to use optimized layer
                            layer["digest"] = serde_json::Value::String(optimized_digest.clone());
                            layer["size"] = serde_json::Value::Number(result.optimized_size.into());
                            optimized = true;

                            info!("Updated manifest to use optimized layer: {} -> {}",
                                digest_str, optimized_digest);
                        }
                    }
                }
            }

            if optimized {
                // Recalculate manifest size and config digest if needed
                info!("Manifest optimized with {} layer optimizations",
                    layers.len());
            }
        }

        Ok(serde_json::to_vec_pretty(&manifest)?)
    }

    /// Get optimization statistics
    pub async fn get_optimization_stats(&self) -> OptimizationStats {
        let cache = self.optimization_cache.read().await;
        let layer_index = self.layer_index.read().await;

        let mut stats = OptimizationStats {
            total_layers: layer_index.total_layers,
            optimized_layers: 0,
            total_original_size: layer_index.total_size_bytes,
            total_optimized_size: layer_index.deduplicated_size_bytes,
            total_savings: 0,
            compression_ratio: 0.0,
            optimization_results: HashMap::new(),
        };

        for result in cache.values() {
            if result.status == OptimizationStatus::Optimized {
                stats.optimized_layers += 1;
                stats.total_savings += result.original_size.saturating_sub(result.optimized_size);
            }

            let type_stats = stats.optimization_results
                .entry(result.optimization_type.clone())
                .or_insert(TypeStats { count: 0, total_savings: 0 });

            type_stats.count += 1;
            type_stats.total_savings += result.original_size.saturating_sub(result.optimized_size);
        }

        if stats.total_original_size > 0 {
            stats.compression_ratio = stats.total_optimized_size as f64 / stats.total_original_size as f64;
        }

        stats
    }

    /// Run background optimization job
    pub async fn run_background_optimization(&self, policy: &OptimizationPolicy) -> Result<()> {
        info!("Starting background optimization job");

        // Find unoptimized layers
        let layer_index = self.layer_index.read().await;
        let unoptimized_layers: Vec<_> = layer_index.layers.values()
            .filter(|layer| layer.optimization_status == OptimizationStatus::Pending)
            .cloned()
            .collect();
        drop(layer_index);

        info!("Found {} layers pending optimization", unoptimized_layers.len());

        // Optimize layers in batches
        for layer in unoptimized_layers {
            // Load layer data
            let key = format!("blobs/{}", layer.digest);
            if let Some(layer_data) = self.storage.get_blob(&key).await? {
                match self.optimize_layer(&layer.digest, &layer_data, policy).await {
                    Ok(result) => {
                        info!("Background optimization completed for {}: {:?}",
                            layer.digest, result.status);
                    }
                    Err(e) => {
                        error!("Background optimization failed for {}: {}",
                            layer.digest, e);
                    }
                }
            }

            // Rate limiting to avoid overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("Background optimization job completed");
        Ok(())
    }

    // Private helper methods

    async fn analyze_layer(&self, layer_data: &[u8]) -> Result<LayerAnalysis> {
        // Simplified layer analysis - in production this would extract and analyze the tar contents
        debug!("Analyzing layer ({} bytes)", layer_data.len());

        // Estimate compression potential based on entropy
        let entropy = self.calculate_entropy(layer_data);
        let compression_potential = (1.0 - entropy).max(0.0);

        Ok(LayerAnalysis {
            content_type: "application/vnd.docker.image.rootfs.diff.tar".to_string(),
            file_count: (layer_data.len() / 4096).min(10000), // Rough estimate
            directory_count: (layer_data.len() / 40960).min(1000), // Rough estimate
            largest_files: vec![], // Would extract from tar in real implementation
            compression_potential,
            duplicate_content_ratio: 0.0, // Would analyze in real implementation
        })
    }

    async fn optimize_compression(&self, data: &[u8], target_compression: &CompressionType) -> Result<Vec<u8>> {
        match target_compression {
            CompressionType::Gzip => {
                use std::io::Write;
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
                encoder.write_all(data)?;
                Ok(encoder.finish()?)
            }
            CompressionType::Zstd => {
                // Would use zstd crate in real implementation
                warn!("Zstd compression not implemented, using gzip");
                Box::pin(self.optimize_compression(data, &CompressionType::Gzip)).await
            }
            CompressionType::Lz4 => {
                // Would use lz4 crate in real implementation
                warn!("LZ4 compression not implemented, using gzip");
                Box::pin(self.optimize_compression(data, &CompressionType::Gzip)).await
            }
            CompressionType::Brotli => {
                // Would use brotli crate in real implementation
                warn!("Brotli compression not implemented, using gzip");
                Box::pin(self.optimize_compression(data, &CompressionType::Gzip)).await
            }
            CompressionType::Uncompressed => Ok(data.to_vec()),
        }
    }

    async fn find_duplicate_layer(&self, layer_data: &[u8]) -> Result<Option<String>> {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(layer_data);
        let content_hash = hex::encode(hasher.finalize());

        let layer_index = self.layer_index.read().await;
        Ok(layer_index.content_map.get(&content_hash).cloned())
    }

    async fn load_layer_index(&self) -> Result<()> {
        debug!("Loading layer index from storage");

        let key = "optimization/layer_index.json";
        if let Some(data) = self.storage.get_blob(key).await? {
            let loaded_index: LayerIndex = serde_json::from_slice(&data)?;
            let mut layer_index = self.layer_index.write().await;
            *layer_index = loaded_index;
            info!("Loaded layer index with {} layers", layer_index.total_layers);
        } else {
            info!("No existing layer index found, starting fresh");
        }

        Ok(())
    }

    async fn update_layer_index(&self, digest: &str, data: &[u8], analysis: &LayerAnalysis) -> Result<()> {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        let content_hash = hex::encode(hasher.finalize());

        let mut layer_index = self.layer_index.write().await;

        let metadata = LayerMetadata {
            digest: digest.to_string(),
            size: data.len() as u64,
            media_type: analysis.content_type.clone(),
            content_hash: content_hash.clone(),
            compression: CompressionType::Gzip, // Assume gzip for now
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            reference_count: 1,
            optimization_status: OptimizationStatus::Optimized,
        };

        layer_index.layers.insert(digest.to_string(), metadata);
        layer_index.content_map.insert(content_hash, digest.to_string());
        layer_index.total_layers += 1;
        layer_index.total_size_bytes += data.len() as u64;

        // Save updated index
        self.save_layer_index(&layer_index).await?;

        Ok(())
    }

    async fn save_layer_index(&self, layer_index: &LayerIndex) -> Result<()> {
        let key = "optimization/layer_index.json";
        let data = serde_json::to_vec(layer_index)?;
        self.storage.put_blob(key, data.into()).await?;
        Ok(())
    }

    async fn increment_layer_references(&self, digest: &str) -> Result<()> {
        let mut layer_index = self.layer_index.write().await;
        if let Some(layer) = layer_index.layers.get_mut(digest) {
            layer.reference_count += 1;
            layer.last_accessed = chrono::Utc::now();
        }
        Ok(())
    }

    async fn get_optimization_result(&self, digest: &str) -> Option<OptimizationResult> {
        let cache = self.optimization_cache.read().await;
        cache.get(digest).cloned()
    }

    async fn cache_optimization_result(&self, digest: &str, result: &OptimizationResult) {
        let mut cache = self.optimization_cache.write().await;
        cache.insert(digest.to_string(), result.clone());
    }

    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut freq = [0u64; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy / 8.0 // Normalize to 0-1 range
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_layers: usize,
    pub optimized_layers: usize,
    pub total_original_size: u64,
    pub total_optimized_size: u64,
    pub total_savings: u64,
    pub compression_ratio: f64,
    pub optimization_results: HashMap<OptimizationType, TypeStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: usize,
    pub total_savings: u64,
}