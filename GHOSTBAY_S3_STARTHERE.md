# 🌊 GhostBay + Drift Integration Guide

**Add GhostBay S3-compatible storage to your Drift OCI registry.**

---

## 🚀 Quick Integration

### 1. Add GhostBay to Drift's Cargo.toml

```toml
[dependencies]
# GhostBay storage engine
ghostbay-engine = { git = "https://github.com/CK-Technology/ghostbay" }
ghostbay-auth = { git = "https://github.com/CK-Technology/ghostbay" }
ghostbay-catalog = { git = "https://github.com/CK-Technology/ghostbay" }

# Required dependencies (if not already present)
tokio = { version = "1.40", features = ["full"] }
axum = "0.7"
bytes = "1.7"
futures = "0.3"
anyhow = "1.0"
```

### 2. Initialize GhostBay Storage in Drift

```rust
use ghostbay_engine::{LocalStorageEngine, StorageEngine};
use ghostbay_auth::AuthService;

// In your Drift main.rs or lib.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize GhostBay storage
    let storage = LocalStorageEngine::new("/var/lib/drift/storage").await?;
    let auth = AuthService::new();

    // Your existing Drift setup...
    let app = create_drift_app_with_ghostbay_storage(storage, auth).await?;

    Ok(())
}
```

### 3. Integrate with Drift's Registry Handlers

```rust
use ghostbay_engine::{StorageEngine, PutObjectRequest, GetObjectRequest};
use bytes::Bytes;
use futures::StreamExt;

// Store container layers
async fn store_layer(
    storage: &dyn StorageEngine,
    digest: &str,
    layer_data: Bytes,
) -> anyhow::Result<String> {
    let request = PutObjectRequest {
        bucket: "drift-registry".to_string(),
        key: format!("blobs/sha256/{}", digest),
        content_type: "application/octet-stream".to_string(),
        content_length: Some(layer_data.len() as u64),
        data: Box::pin(futures::stream::once(async { Ok(layer_data) })),
    };

    storage.put_object(request).await
}

// Retrieve container layers
async fn get_layer(
    storage: &dyn StorageEngine,
    digest: &str,
) -> anyhow::Result<Option<Bytes>> {
    let request = GetObjectRequest {
        bucket: "drift-registry".to_string(),
        key: format!("blobs/sha256/{}", digest),
        range: None,
    };

    if let Some(response) = storage.get_object(request).await? {
        let mut data = Vec::new();
        let mut stream = response.data;
        while let Some(chunk) = stream.next().await {
            data.extend_from_slice(&chunk?);
        }
        Ok(Some(data.into()))
    } else {
        Ok(None)
    }
}

// Store manifests
async fn store_manifest(
    storage: &dyn StorageEngine,
    repo: &str,
    reference: &str,
    manifest: &[u8],
) -> anyhow::Result<String> {
    let request = PutObjectRequest {
        bucket: "drift-registry".to_string(),
        key: format!("manifests/{}/{}", repo, reference),
        content_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
        content_length: Some(manifest.len() as u64),
        data: Box::pin(futures::stream::once(async { Ok(manifest.to_vec().into()) })),
    };

    storage.put_object(request).await
}
```

### 4. Handle Large Uploads with Multipart

```rust
use ghostbay_engine::{CreateMultipartUploadRequest, UploadPartRequest, CompleteMultipartUploadRequest};

async fn store_large_layer(
    storage: &dyn StorageEngine,
    digest: &str,
    data: Bytes,
) -> anyhow::Result<String> {
    const MULTIPART_THRESHOLD: usize = 100 * 1024 * 1024; // 100MB

    if data.len() > MULTIPART_THRESHOLD {
        // Use multipart upload for large layers
        let upload_id = storage.create_multipart_upload(CreateMultipartUploadRequest {
            bucket: "drift-registry".to_string(),
            key: format!("blobs/sha256/{}", digest),
            content_type: "application/octet-stream".to_string(),
            metadata: None,
        }).await?;

        // Upload in 50MB chunks
        const CHUNK_SIZE: usize = 50 * 1024 * 1024;
        let mut parts = Vec::new();

        for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let part_number = (i + 1) as i32;
            let etag = storage.upload_part(UploadPartRequest {
                bucket: "drift-registry".to_string(),
                key: format!("blobs/sha256/{}", digest),
                upload_id: upload_id.clone(),
                part_number,
                data: Box::pin(futures::stream::once(async { Ok(chunk.to_vec().into()) })),
            }).await?;

            parts.push(ghostbay_engine::MultipartUploadPart {
                part_number,
                etag,
                size: chunk.len() as u64,
            });
        }

        storage.complete_multipart_upload(CompleteMultipartUploadRequest {
            bucket: "drift-registry".to_string(),
            key: format!("blobs/sha256/{}", digest),
            upload_id,
            parts,
        }).await
    } else {
        // Regular upload for smaller layers
        store_layer(storage, digest, data).await
    }
}
```

---

## 🔧 Configuration

### Drift Configuration with GhostBay

```toml
# drift.toml
[server]
host = "0.0.0.0"
api_port = 5000
ui_port = 5001

[storage]
backend = "ghostbay"
path = "/var/lib/drift/storage"
bucket = "drift-registry"

[auth]
# Integrate with GhostBay auth if needed
enabled = true
```

### Example Drift Service Struct

```rust
use std::sync::Arc;
use ghostbay_engine::LocalStorageEngine;
use ghostbay_auth::AuthService;

#[derive(Clone)]
pub struct DriftState {
    pub storage: Arc<LocalStorageEngine>,
    pub auth: Arc<AuthService>,
}

impl DriftState {
    pub async fn new(storage_path: &str) -> anyhow::Result<Self> {
        let storage = Arc::new(LocalStorageEngine::new(storage_path).await?);
        let auth = Arc::new(AuthService::new());

        Ok(Self { storage, auth })
    }
}
```

---

## 🐳 Docker Integration

### Dockerfile for Drift + GhostBay

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build with GhostBay integration
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/drift /usr/local/bin/

EXPOSE 5000 5001

CMD ["drift"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  drift:
    build: .
    ports:
      - "5000:5000"  # Registry API
      - "5001:5001"  # Web UI
    volumes:
      - drift-storage:/var/lib/drift/storage
    environment:
      - DRIFT_STORAGE_PATH=/var/lib/drift/storage
      - RUST_LOG=info

volumes:
  drift-storage:
```

---

## 🧪 Testing the Integration

```bash
# Build and run Drift with GhostBay
cargo run --release

# Test with Docker
docker pull alpine:latest
docker tag alpine:latest localhost:5000/alpine:latest
docker push localhost:5000/alpine:latest

# Test with Podman
podman pull alpine:latest
podman tag alpine:latest localhost:5000/test/alpine:latest
podman push localhost:5000/test/alpine:latest
```

---

## 📊 Performance Tips

1. **Enable compression** for manifest storage
2. **Use streaming** for large layer uploads/downloads
3. **Implement caching** for frequently accessed layers
4. **Leverage multipart uploads** for layers > 100MB

---

## 🆘 Quick Troubleshooting

**Build errors:**
```bash
# Update Cargo.lock
cargo update

# Clean build
cargo clean && cargo build
```

**Storage issues:**
```bash
# Check permissions
ls -la /var/lib/drift/storage

# Create directory
mkdir -p /var/lib/drift/storage
```

Ready to use GhostBay as your Drift storage backend! 🚀