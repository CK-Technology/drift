# QUIC Transport

Drift Registry includes next-generation QUIC transport for high-performance, low-latency container image transfers. QUIC provides significant advantages over traditional HTTP/2, especially for registry operations.

## 🌐 Overview

### What is QUIC?

QUIC (Quick UDP Internet Connections) is a modern transport protocol that provides:

- **Reduced latency**: 0-RTT connection establishment
- **Multiplexing**: Multiple streams without head-of-line blocking
- **Built-in encryption**: TLS 1.3 integrated into the transport
- **Connection migration**: Robust against network changes
- **Improved performance**: Better handling of packet loss

### Benefits for Container Registries

- **Faster image pulls**: Reduced connection setup time
- **Parallel layer downloads**: Multiple layers simultaneously
- **Better mobile/edge performance**: Resilient to network changes
- **Enhanced security**: Always encrypted, forward secrecy

## 🛠️ Supported Backends

Drift supports multiple QUIC implementations:

### Quinn (Recommended)
- **Type**: Pure Rust, async-first implementation
- **Features**: Full QUIC support, 0-RTT, HTTP/3 ready
- **Best for**: Production deployments, maximum performance

### Quiche
- **Type**: Google's QUIC implementation (Rust bindings)
- **Features**: Battle-tested, used in Chrome/Chromium
- **Best for**: Compatibility with existing QUIC infrastructure

### Custom gquic
- **Type**: Your custom QUIC library integration
- **Features**: Tailored for specific requirements
- **Best for**: Specialized deployments, custom optimizations

## ⚙️ Configuration

### Basic QUIC Setup

```toml
[quic]
enabled = true
backend = "quinn"  # or "quiche", "gquic"
bind_addr = "0.0.0.0:5443"

# TLS certificates (required)
cert_path = "./certs/server.crt"
key_path = "./certs/server.key"

# Performance tuning
max_connections = 1000
max_idle_timeout_ms = 60000
keep_alive_interval_ms = 30000

# Protocol configuration
application_protocols = ["drift-registry"]
enable_0rtt = true
enable_early_data = true
```

### Advanced Configuration

```toml
[quic]
enabled = true
backend = "quinn"

# Network configuration
bind_addr = "0.0.0.0:5443"
max_connections = 2000
max_concurrent_streams = 100

# Timeout settings
max_idle_timeout_ms = 300000     # 5 minutes
keep_alive_interval_ms = 15000   # 15 seconds
connection_timeout_ms = 10000    # 10 seconds

# Buffer sizes
send_buffer_size = 1048576       # 1 MB
receive_buffer_size = 1048576    # 1 MB
max_datagram_size = 1350         # Standard MTU

# TLS configuration
cert_chain = []  # DER encoded cert chain
private_key = [] # DER encoded private key
cert_path = "./certs/server.crt"
key_path = "./certs/server.key"

# Protocol features
application_protocols = ["drift-registry", "h3"]
enable_0rtt = true
enable_early_data = true
validate_peer_certificates = true

# Performance optimizations
congestion_control = "cubic"     # or "bbr"
initial_window_size = 32768
max_window_size = 16777216
ack_delay_exponent = 3
```

### Environment Variables

```bash
# Enable QUIC
export DRIFT_QUIC_ENABLED=true
export DRIFT_QUIC_BACKEND=quinn

# Network settings
export DRIFT_QUIC_BIND_ADDR=0.0.0.0:5443
export DRIFT_QUIC_MAX_CONNECTIONS=1000

# TLS certificates
export DRIFT_QUIC_CERT_PATH=/path/to/cert.pem
export DRIFT_QUIC_KEY_PATH=/path/to/key.pem

# Performance tuning
export DRIFT_QUIC_IDLE_TIMEOUT=60000
export DRIFT_QUIC_KEEP_ALIVE=30000
```

## 🚀 Backend-Specific Configuration

### Quinn Backend

```toml
[quic]
backend = "quinn"

# Quinn-specific settings
[quic.quinn]
# Connection configuration
max_concurrent_bidi_streams = 1000
max_concurrent_uni_streams = 1000
datagram_receive_buffer_size = 65536
datagram_send_buffer_size = 65536

# Transport parameters
initial_rtt = 333000  # 333ms
max_ack_delay = 25000 # 25ms
ack_delay_exponent = 3
max_udp_payload_size = 1350

# Congestion control
congestion_controller = "cubic"  # or "bbr"
initial_window = 32768
minimum_window = 2048
loss_detection_timer = 25000

# Keep-alive
keep_alive_interval = 30000
max_idle_timeout = 60000

# 0-RTT configuration
enable_0rtt = true
max_0rtt_bytes = 8192
```

### Quiche Backend

```toml
[quic]
backend = "quiche"

# Quiche-specific settings
[quic.quiche]
# Protocol configuration
max_recv_udp_payload_size = 1350
max_send_udp_payload_size = 1350
initial_max_data = 10000000
initial_max_stream_data_bidi_local = 1000000
initial_max_stream_data_bidi_remote = 1000000
initial_max_streams_bidi = 100

# Congestion control
cc_algorithm = "cubic"  # or "reno", "bbr"
hystart = true
pacing = true

# Loss recovery
max_ack_delay = 25
ack_delay_exponent = 3
time_threshold = 1.125
packet_threshold = 3

# Connection migration
disable_active_migration = false
enable_path_challenge = true
```

### Custom gquic Backend

```toml
[quic]
backend = "gquic"

# gquic-specific settings
[quic.gquic]
# Custom library configuration
library_path = "/path/to/libgquic.so"
config_file = "/path/to/gquic.conf"

# Protocol parameters
version = "draft-34"
connection_id_length = 8
stateless_reset_token_length = 16

# Performance settings
max_packet_size = 1350
send_window = 1048576
receive_window = 1048576
max_streams = 1000

# Custom features
enable_custom_congestion_control = true
enable_packet_pacing = true
enable_fec = false  # Forward Error Correction
```

## 📡 API Integration

### QUIC Management API

```bash
# Get QUIC status
curl https://registry.example.com/api/quic/status

# Response
{
  "enabled": true,
  "backend": "quinn",
  "bind_addr": "0.0.0.0:5443",
  "active_connections": 15,
  "supported_features": [
    "ping", "blob-transfer", "manifest-transfer",
    "quinn-backend", "0rtt"
  ]
}
```

### Connection Testing

```bash
# Ping QUIC endpoint
curl -X POST https://registry.example.com/api/quic/ping/192.168.1.100:5443

# Test blob transfer
curl -X POST https://registry.example.com/api/quic/test/blob/sha256:abc123 \
  -H "Content-Type: application/json" \
  -d '{"target_addr": "192.168.1.100:5443"}'

# Test manifest transfer
curl -X POST https://registry.example.com/api/quic/test/manifest/hello-world:latest \
  -H "Content-Type: application/json" \
  -d '{"target_addr": "192.168.1.100:5443"}'
```

### Statistics and Monitoring

```bash
# Get QUIC statistics
curl https://registry.example.com/api/quic/stats

# Response
{
  "active_connections": 15,
  "backend": 1,
  "bytes_sent": 1048576000,
  "bytes_received": 2097152000,
  "packets_sent": 1000000,
  "packets_received": 1500000,
  "connection_errors": 5,
  "stream_errors": 2
}
```

## 🔧 TLS Certificate Setup

### Self-Signed Certificates

```bash
# Generate QUIC-compatible certificates
openssl genrsa -out quic-server.key 4096

# Create certificate with SAN for QUIC
openssl req -new -x509 -sha256 -key quic-server.key -out quic-server.crt -days 365 \
  -config <(cat <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req

[req_distinguished_name]
CN = registry.example.com

[v3_req]
keyUsage = keyEncipherment, dataEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = registry.example.com
DNS.2 = localhost
IP.1 = 127.0.0.1
IP.2 = ::1
EOF
)

# Update configuration
[quic]
cert_path = "./certs/quic-server.crt"
key_path = "./certs/quic-server.key"
```

### Let's Encrypt with DNS Challenge

```bash
# Use certbot with DNS challenge for QUIC
certbot certonly \
  --dns-cloudflare \
  --dns-cloudflare-credentials ~/.secrets/cloudflare.ini \
  -d registry.example.com \
  --preferred-challenges dns-01

# Configure auto-renewal
crontab -e
# Add: 0 0 * * * certbot renew --post-hook "systemctl reload drift-registry"
```

## 🚀 Client Integration

### Drift CLI Client with QUIC

```bash
# Use QUIC for registry operations
drift pull quic://registry.example.com:5443/hello-world:latest

# Push with QUIC
drift push quic://registry.example.com:5443/my-image:v1.0

# Configure QUIC as preferred transport
drift config set transport.preferred quic
drift config set transport.quic.verify_certificates true
```

### Custom Client Implementation

```rust
use drift_client::QuicRegistryClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create QUIC client
    let client = QuicRegistryClient::builder()
        .endpoint("registry.example.com:5443")
        .verify_certificates(true)
        .build()
        .await?;

    // Pull manifest
    let manifest = client.get_manifest("hello-world", "latest").await?;
    println!("Manifest: {}", manifest);

    // Pull blob
    let blob = client.get_blob(&manifest.layers[0].digest).await?;
    println!("Blob size: {} bytes", blob.len());

    Ok(())
}
```

## 📊 Performance Optimization

### Network Tuning

```bash
# System-level UDP buffer optimization
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.netdev_max_backlog = 5000' >> /etc/sysctl.conf
echo 'net.ipv4.udp_mem = 102400 873800 16777216' >> /etc/sysctl.conf
sysctl -p
```

### Application Tuning

```toml
[quic]
# Optimize for high-throughput
max_concurrent_streams = 1000
send_buffer_size = 4194304      # 4 MB
receive_buffer_size = 4194304   # 4 MB
initial_window_size = 1048576   # 1 MB

# Reduce latency
max_ack_delay = 10              # 10ms
keep_alive_interval_ms = 5000   # 5 seconds

# Congestion control
congestion_control = "bbr"      # Better for high-bandwidth
pacing = true
hystart = true
```

### Connection Pooling

```toml
[quic]
# Connection management
max_connections = 2000
connection_pool_size = 100
idle_connection_timeout = 300000  # 5 minutes
connection_reuse = true

# Stream management
max_streams_per_connection = 100
stream_timeout_ms = 30000
stream_keep_alive = true
```

## 📈 Monitoring and Metrics

### Prometheus Metrics

```yaml
# Available QUIC metrics
drift_quic_connections_active{backend="quinn"}
drift_quic_connections_total{backend="quinn"}
drift_quic_bytes_sent_total{backend="quinn"}
drift_quic_bytes_received_total{backend="quinn"}
drift_quic_packets_sent_total{backend="quinn"}
drift_quic_packets_received_total{backend="quinn"}
drift_quic_connection_errors_total{backend="quinn"}
drift_quic_stream_errors_total{backend="quinn"}
drift_quic_handshake_duration_seconds{backend="quinn"}
drift_quic_rtt_seconds{backend="quinn"}
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Drift Registry QUIC Metrics",
    "panels": [
      {
        "title": "Active QUIC Connections",
        "type": "graph",
        "targets": [
          {
            "expr": "drift_quic_connections_active",
            "legendFormat": "{{backend}}"
          }
        ]
      },
      {
        "title": "QUIC Throughput",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(drift_quic_bytes_sent_total[5m])",
            "legendFormat": "Sent - {{backend}}"
          },
          {
            "expr": "rate(drift_quic_bytes_received_total[5m])",
            "legendFormat": "Received - {{backend}}"
          }
        ]
      }
    ]
  }
}
```

## 🚨 Troubleshooting

### Common Issues

**"QUIC connection failed"**
```bash
# Check firewall rules
ufw allow 5443/udp

# Verify certificate
openssl x509 -in certs/server.crt -text -noout

# Test UDP connectivity
nc -u registry.example.com 5443
```

**"Certificate verification failed"**
```bash
# Check certificate validity
openssl verify -CAfile ca.crt server.crt

# Verify SAN entries
openssl x509 -in server.crt -text | grep -A 1 "Subject Alternative Name"
```

**"High packet loss"**
```bash
# Check network statistics
netstat -su | grep -i udp

# Monitor QUIC metrics
curl https://registry.example.com/api/quic/stats
```

### Debug Configuration

```toml
[logging]
level = "debug"
quic_debug = true

[quic]
# Enable detailed logging
log_packets = true
log_frames = true
log_congestion_control = true
```

### Performance Issues

```bash
# Monitor system resources
htop
iotop
nethogs

# Check QUIC-specific metrics
curl https://registry.example.com/api/quic/stats

# Profile application
perf record -g target/release/drift server
perf report
```

## 🔄 Migration Guide

### From HTTP/2 to QUIC

1. **Deploy QUIC alongside HTTP/2**:
   ```toml
   [server]
   bind_addr = "0.0.0.0:5000"  # HTTP/2

   [quic]
   enabled = true
   bind_addr = "0.0.0.0:5443"  # QUIC
   ```

2. **Test QUIC functionality**:
   ```bash
   curl https://registry.example.com/api/quic/status
   ```

3. **Update clients gradually**:
   ```bash
   # Configure clients to prefer QUIC
   drift config set transport.preferred quic
   ```

4. **Monitor performance**:
   - Compare transfer speeds
   - Monitor error rates
   - Check resource usage

### Performance Comparison

| Metric | HTTP/2 | QUIC | Improvement |
|--------|--------|------|-------------|
| Connection Setup | 2-3 RTT | 0-1 RTT | 60-80% faster |
| Parallel Streams | Limited by TCP | Native | 2-3x throughput |
| Mobile Performance | Poor | Excellent | 5-10x better |
| Security | TLS 1.2/1.3 | TLS 1.3 built-in | Always encrypted |

## 📞 Support

### QUIC Resources
- [QUIC Specification](https://tools.ietf.org/html/draft-ietf-quic-transport)
- [Quinn Documentation](https://docs.rs/quinn/)
- [Quiche Documentation](https://docs.rs/quiche/)

### Drift Registry
- [QUIC Issues](https://github.com/CK-Technology/drift/issues?q=label%3Aquic)
- [Performance Discussions](https://github.com/CK-Technology/drift/discussions/categories/performance)

---

**Next Steps**: [Content Signing](./content-signing.md) | [Image Optimization](./image-optimization.md) | [Monitoring Setup](../operations/monitoring.md)