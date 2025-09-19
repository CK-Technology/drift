# Drift Registry Installation Guide

This guide covers multiple installation methods for the Drift Container Registry, from quick development setups to production deployments.

## 🚀 Quick Start

### Option 1: Docker Compose (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/CK-Technology/drift.git
cd drift

# Start with Docker Compose
docker-compose up -d

# Registry will be available at:
# - API: http://localhost:5000
# - Web UI: http://localhost:5001
```

### Option 2: Pre-built Binary

```bash
# Download latest release
curl -L https://github.com/CK-Technology/drift/releases/latest/download/drift-linux-amd64.tar.gz | tar xz

# Run with default configuration
./drift server

# Or with custom config
./drift server --config drift.toml
```

### Option 3: Container Image

```bash
# Run with filesystem storage
docker run -d \
  --name drift-registry \
  -p 5000:5000 \
  -p 5001:5001 \
  -v ./data:/app/data \
  drift:latest

# Run with configuration file
docker run -d \
  --name drift-registry \
  -p 5000:5000 \
  -p 5001:5001 \
  -v ./drift.toml:/app/drift.toml \
  -v ./data:/app/data \
  drift:latest --config /app/drift.toml
```

## 🛠️ Building from Source

### Prerequisites

- **Rust 1.70+** with 2024 edition support
- **Node.js 18+** (for web UI)
- **Git**

### Build Steps

```bash
# Clone repository
git clone https://github.com/CK-Technology/drift.git
cd drift

# Build with all features
cargo build --release --all-features

# Build without optional features
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

### Feature Flags

```bash
# Build with specific features
cargo build --release --features "bolt-integration,quinn-quic"

# Available features:
# - bolt-integration: Bolt protocol support
# - quinn-quic: Quinn QUIC backend
# - quiche-quic: Quiche QUIC backend
# - gquic: Custom gquic backend
# - ghostbay-storage: GhostBay storage backend
```

## ⚙️ Configuration

### Basic Configuration

Create `drift.toml`:

```toml
[server]
bind_addr = "0.0.0.0:5000"
ui_addr = "0.0.0.0:5001"
workers = 4

[storage]
type = "filesystem"
path = "./data"

[auth]
mode = "basic"
jwt_secret = "change-me-in-production"

[auth.basic]
users = ["admin:changeme123"]

[registry]
max_upload_size_mb = 1000
rate_limit_per_hour = 1000
```

### Production Configuration

```toml
[server]
bind_addr = "0.0.0.0:5000"
ui_addr = "0.0.0.0:5001"
workers = 8
max_connections = 2000

[storage]
type = "s3"

[storage.s3]
endpoint = "https://s3.amazonaws.com"
region = "us-west-2"
bucket = "my-registry-bucket"
access_key = "AKIAIOSFODNN7EXAMPLE"
secret_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"

[auth]
mode = "oidc"
jwt_secret = "your-256-bit-secret-key"
token_expiry_hours = 8

[auth.oauth.azure]
client_id = "your-azure-client-id"
client_secret = "your-azure-client-secret"
tenant_id = "your-tenant-id"
enabled = true

[garbage_collector]
enabled = true
interval_hours = 24
grace_period_hours = 168

[bolt]
enable_profile_validation = true
enable_plugin_sandbox = true

[quic]
enabled = true
backend = "quinn"
bind_addr = "0.0.0.0:5443"
cert_path = "./certs/server.crt"
key_path = "./certs/server.key"

[signing]
enabled = true
signature_formats = ["cosign", "notary-v2"]

[optimization]
enabled = true
background_optimization = true
preferred_compression = "zstd"
```

## 🐳 Docker Deployment

### Docker Compose (Production)

```yaml
version: '3.8'

services:
  drift-registry:
    image: drift:latest
    container_name: drift-registry
    restart: unless-stopped
    ports:
      - "5000:5000"  # Registry API
      - "5001:5001"  # Web UI
      - "5443:5443"  # QUIC (optional)
    environment:
      - DRIFT_AUTH_MODE=oidc
      - DRIFT_AUTH_JWT_SECRET=${JWT_SECRET}
      - DRIFT_AZURE_CLIENT_ID=${AZURE_CLIENT_ID}
      - DRIFT_AZURE_CLIENT_SECRET=${AZURE_CLIENT_SECRET}
      - DRIFT_AZURE_TENANT_ID=${AZURE_TENANT_ID}
      - DRIFT_S3_ACCESS_KEY=${S3_ACCESS_KEY}
      - DRIFT_S3_SECRET_KEY=${S3_SECRET_KEY}
    volumes:
      - ./drift.toml:/app/drift.toml:ro
      - ./certs:/app/certs:ro
      - drift-data:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    depends_on:
      - redis
      - postgres

  redis:
    image: redis:alpine
    container_name: drift-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data

  postgres:
    image: postgres:15
    container_name: drift-postgres
    restart: unless-stopped
    environment:
      - POSTGRES_DB=drift
      - POSTGRES_USER=drift
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres-data:/var/lib/postgresql/data

volumes:
  drift-data:
  redis-data:
  postgres-data:
```

### Environment Variables

```bash
# Create .env file
cat > .env << EOF
JWT_SECRET=$(openssl rand -base64 32)
AZURE_CLIENT_ID=your-azure-client-id
AZURE_CLIENT_SECRET=your-azure-client-secret
AZURE_TENANT_ID=your-tenant-id
S3_ACCESS_KEY=your-s3-access-key
S3_SECRET_KEY=your-s3-secret-key
POSTGRES_PASSWORD=$(openssl rand -base64 16)
EOF

# Start services
docker-compose --env-file .env up -d
```

## ☸️ Kubernetes Deployment

### Helm Chart

```bash
# Add Drift Helm repository
helm repo add drift https://ck-technology.github.io/drift-helm
helm repo update

# Install with custom values
helm install drift-registry drift/drift \
  --namespace drift-registry \
  --create-namespace \
  --values values.yaml
```

### Manual Kubernetes Deployment

```yaml
# drift-registry.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: drift-registry
  namespace: drift-registry
spec:
  replicas: 2
  selector:
    matchLabels:
      app: drift-registry
  template:
    metadata:
      labels:
        app: drift-registry
    spec:
      containers:
      - name: drift-registry
        image: drift:latest
        ports:
        - containerPort: 5000
          name: api
        - containerPort: 5001
          name: ui
        env:
        - name: DRIFT_AUTH_MODE
          value: "oidc"
        - name: DRIFT_AUTH_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: drift-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /app/drift.toml
          subPath: drift.toml
        - name: certs
          mountPath: /app/certs
        livenessProbe:
          httpGet:
            path: /health
            port: 5000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /readyz
            port: 5000
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: drift-config
      - name: certs
        secret:
          secretName: drift-tls

---
apiVersion: v1
kind: Service
metadata:
  name: drift-registry
  namespace: drift-registry
spec:
  selector:
    app: drift-registry
  ports:
  - name: api
    port: 5000
    targetPort: 5000
  - name: ui
    port: 5001
    targetPort: 5001

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: drift-registry
  namespace: drift-registry
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/proxy-body-size: "0"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "600"
spec:
  tls:
  - hosts:
    - registry.example.com
    secretName: drift-tls
  rules:
  - host: registry.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: drift-registry
            port:
              number: 5000
```

## 🔧 System Requirements

### Minimum Requirements

- **CPU**: 2 cores
- **Memory**: 4 GB RAM
- **Storage**: 20 GB (plus container image storage)
- **Network**: 1 Gbps

### Recommended (Production)

- **CPU**: 8+ cores
- **Memory**: 16+ GB RAM
- **Storage**: 500+ GB SSD (or object storage)
- **Network**: 10 Gbps

### High Availability Setup

- **Nodes**: 3+ registry instances
- **Load Balancer**: External load balancer
- **Database**: PostgreSQL cluster
- **Storage**: Distributed object storage (S3, etc.)
- **Cache**: Redis cluster

## 🔐 TLS/SSL Configuration

### Generate Self-Signed Certificates

```bash
# Create certificates directory
mkdir -p certs

# Generate private key
openssl genrsa -out certs/server.key 4096

# Generate certificate
openssl req -new -x509 -sha256 -key certs/server.key -out certs/server.crt -days 365 \
  -subj "/C=US/ST=CA/L=San Francisco/O=Example/OU=IT/CN=registry.example.com"

# Update configuration
echo "cert_path = \"./certs/server.crt\"" >> drift.toml
echo "key_path = \"./certs/server.key\"" >> drift.toml
```

### Let's Encrypt with Cert-Manager

```yaml
# cert-manager-issuer.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

## 🧪 Verification

### Test Installation

```bash
# Check health endpoint
curl http://localhost:5000/health

# Check readiness
curl http://localhost:5000/readyz

# Test Docker login
docker login localhost:5000

# Push test image
docker tag hello-world localhost:5000/test/hello-world
docker push localhost:5000/test/hello-world

# Verify catalog
curl http://localhost:5000/v2/_catalog
```

### Web UI Access

Visit `http://localhost:5001` to access the web interface:

- **Login page**: Authentication
- **Browse repositories**: View container images
- **Search functionality**: Find images and tags
- **Admin panel**: System administration

## 🚨 Troubleshooting

### Common Issues

**"Connection refused"**
```bash
# Check if service is running
systemctl status drift-registry

# Check port bindings
netstat -tlnp | grep :5000

# Check firewall
ufw status
```

**"Permission denied"**
```bash
# Check file permissions
ls -la drift.toml
chmod 600 drift.toml

# Check data directory
chown -R drift:drift /app/data
```

**"Out of disk space"**
```bash
# Check disk usage
df -h

# Clean up old images
drift gc --dry-run
drift gc
```

### Logs

```bash
# View logs (systemd)
journalctl -u drift-registry -f

# View logs (Docker)
docker logs -f drift-registry

# Enable debug logging
export RUST_LOG=debug
drift server
```

## 📞 Support

- **Documentation**: [docs/](../docs/)
- **GitHub Issues**: [Issues](https://github.com/CK-Technology/drift/issues)
- **Discussions**: [Discussions](https://github.com/CK-Technology/drift/discussions)

---

**Next Steps**: [Configuration Guide](./configuration.md) | [SSO Setup](./sso/README.md) | [Production Deployment](./deployment/production.md)