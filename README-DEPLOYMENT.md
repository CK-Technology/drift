# 🌊 Drift Registry - Deployment Guide

This guide covers deploying the Drift Registry in various environments.

## 🚀 Quick Start with Docker Compose

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- 10GB+ storage

### 1. Clone and Setup

```bash
git clone https://github.com/your-org/drift.git
cd drift
```

### 2. Environment Configuration

Copy the example configuration:

```bash
cp drift.toml.example drift.toml
```

Edit `drift.toml` for your environment:

```toml
[auth.basic]
users = [
    "admin:your-secure-password",
    "ci:your-ci-token"
]

[storage]
backend = "s3"  # Use MinIO

[storage.s3]
endpoint = "http://minio:9000"
bucket = "drift-registry"
access_key = "your-access-key"
secret_key = "your-secret-key"
```

### 3. Start Services

```bash
# Start all services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f drift
```

### 4. Access the Services

- **Registry API**: http://localhost:5000/v2/
- **Web UI**: http://localhost:5001/
- **MinIO Console**: http://localhost:9001/
- **Grafana**: http://localhost:3000/ (admin/grafana123)

## 🔧 Configuration Options

### Storage Backends

#### Filesystem Storage
```toml
[storage]
backend = "fs"
path = "/var/lib/drift"
```

#### S3/MinIO Storage
```toml
[storage]
backend = "s3"

[storage.s3]
endpoint = "https://s3.amazonaws.com"
region = "us-east-1"
bucket = "my-drift-registry"
access_key = "AKIAIOSFODNN7EXAMPLE"
secret_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
path_style = false
```

#### GhostBay Storage (when available)
```toml
[storage]
backend = "ghostbay"

[storage.ghostbay]
endpoint = "http://ghostbay:8080"
bucket = "drift-registry"
```

### Authentication

#### Basic Authentication
```toml
[auth]
mode = "basic"

[auth.basic]
users = ["admin:password", "user:pass"]
```

#### OIDC Authentication
```toml
[auth]
mode = "oidc"

[auth.oidc]
issuer = "https://auth.example.com/realms/main"
client_id = "drift"
client_secret = "your-client-secret"
```

## 🐳 Using the Registry

### Docker Commands

```bash
# Login
docker login localhost:5000 -u admin -p changeme

# Tag and push
docker tag alpine:latest localhost:5000/test/alpine:latest
docker push localhost:5000/test/alpine:latest

# Pull
docker pull localhost:5000/test/alpine:latest
```

### Podman Commands

```bash
# Login
podman login localhost:5000 -u admin -p changeme

# Push
podman push localhost:5000/test/alpine:latest

# Pull
podman pull localhost:5000/test/alpine:latest
```

### Bolt Commands (when available)

```bash
# Configure registry
bolt config set registry.url http://localhost:5000
bolt config set registry.username admin
bolt config set registry.password changeme

# Push profile
bolt profiles push my-gaming-profile

# Pull profile
bolt profiles pull steam-gaming
```

## 🔒 Production Deployment

### 1. Security Configuration

```bash
# Generate secure passwords
openssl rand -base64 32

# Generate JWT secret
openssl rand -hex 64
```

### 2. TLS/SSL Setup

```bash
# Generate self-signed certificate (for testing)
openssl req -x509 -newkey rsa:4096 -keyout ssl/drift.key -out ssl/drift.crt -days 365 -nodes

# Or use Let's Encrypt
certbot certonly --standalone -d registry.yourdomain.com
```

### 3. Environment Variables

```bash
# Production overrides
export DRIFT_AUTH_JWT_SECRET="your-64-char-hex-secret"
export DRIFT_S3_ACCESS_KEY="your-production-access-key"
export DRIFT_S3_SECRET_KEY="your-production-secret-key"
```

### 4. Database Setup (Optional)

For production metadata storage:

```sql
-- PostgreSQL setup
CREATE DATABASE drift;
CREATE USER drift WITH PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE drift TO drift;
```

### 5. Monitoring Setup

The stack includes Prometheus and Grafana for monitoring:

```bash
# Access Grafana
open http://localhost:3000
# Login: admin / grafana123

# Import Drift dashboard
# Dashboard ID: (custom dashboard included)
```

## 🔧 Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Check file permissions
ls -la /var/lib/drift/
sudo chown -R 1000:1000 /var/lib/drift/
```

#### Storage Issues
```bash
# Check MinIO connectivity
docker compose exec drift curl -f http://minio:9000/minio/health/live

# Check bucket creation
docker compose exec minio-init mc ls myminio/
```

#### Registry Not Accessible
```bash
# Check service status
docker compose ps
docker compose logs drift

# Test API endpoint
curl -f http://localhost:5000/v2/
```

### Log Locations

```bash
# Application logs
docker compose logs drift

# Nginx logs
docker compose logs nginx

# MinIO logs
docker compose logs minio
```

## 📊 Performance Tuning

### Resource Allocation

```yaml
# docker-compose.override.yml
services:
  drift:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 2G
```

### Caching Configuration

```toml
# drift.toml
[cache]
enabled = true
redis_url = "redis://redis:6379/0"
ttl_seconds = 3600
```

## 🚀 Scaling

### Horizontal Scaling

```yaml
# docker-compose.scale.yml
services:
  drift:
    deploy:
      replicas: 3

  nginx:
    depends_on:
      - drift
    # Load balancer configuration
```

### Load Balancing

Update nginx.conf for multiple instances:

```nginx
upstream drift_api {
    server drift_1:5000;
    server drift_2:5000;
    server drift_3:5000;
    keepalive 32;
}
```

## 🔄 Backup and Recovery

### Database Backup

```bash
# PostgreSQL backup
docker compose exec postgres pg_dump -U drift drift > backup.sql

# Restore
docker compose exec -T postgres psql -U drift drift < backup.sql
```

### Storage Backup

```bash
# Filesystem backup
tar -czf drift-backup.tar.gz /var/lib/drift/

# S3 backup (using MinIO client)
docker compose exec minio-init mc mirror myminio/drift-registry /backup/
```

## 📈 Monitoring and Alerting

### Key Metrics

- Registry API response time
- Storage usage
- Authentication failures
- Upload/download rates
- Error rates

### Alerts

Configure alerts in Grafana for:
- High error rates (>5%)
- Storage usage (>80%)
- API latency (>500ms)
- Service downtime

## 🔗 Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml
- name: Build and push
  run: |
    docker build -t localhost:5000/myapp:${{ github.sha }} .
    docker push localhost:5000/myapp:${{ github.sha }}
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: drift-registry
spec:
  replicas: 3
  selector:
    matchLabels:
      app: drift-registry
  template:
    metadata:
      labels:
        app: drift-registry
    spec:
      containers:
      - name: drift
        image: drift:latest
        ports:
        - containerPort: 5000
        - containerPort: 5001
```

Ready to deploy your Drift Registry! 🌊