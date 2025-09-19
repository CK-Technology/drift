# Drift Registry Documentation

Welcome to the Drift Registry documentation! Drift is a modern, high-performance OCI-compliant container registry with advanced features for gaming, security, and optimization.

## 📚 Documentation Structure

### Quick Start
- [Installation Guide](./installation.md) - Get Drift up and running
- [Configuration Reference](./configuration.md) - Complete configuration options
- [API Reference](./api/README.md) - REST API documentation

### Core Features
- [OCI Registry](./features/oci-registry.md) - Docker/OCI compliance and usage
- [Bolt Integration](./features/bolt-integration.md) - Gaming-optimized containers
- [QUIC Transport](./features/quic-transport.md) - High-performance communication
- [Content Signing](./features/content-signing.md) - Image signing and verification
- [Image Optimization](./features/image-optimization.md) - Automated optimization

### Authentication & Security
- [SSO Configuration](./sso/README.md) - Single Sign-On setup guides
- [Authentication](./auth/authentication.md) - Auth modes and configuration
- [Security Best Practices](./security/best-practices.md) - Production security

### Operations
- [Deployment](./deployment/README.md) - Production deployment guides
- [Monitoring](./operations/monitoring.md) - Metrics and observability
- [Backup & Recovery](./operations/backup.md) - Data protection
- [Troubleshooting](./operations/troubleshooting.md) - Common issues

### Development
- [Architecture](./development/architecture.md) - System design overview
- [Contributing](./development/contributing.md) - How to contribute
- [API Development](./development/api.md) - Extending the API

## 🚀 Key Features

### Performance
- **QUIC Transport**: Low-latency, multiplexed communication
- **Image Optimization**: Automated layer deduplication and compression
- **Async I/O**: Full async architecture with Tokio

### Gaming Integration
- **Bolt Protocol**: Native support for gaming-optimized containers
- **Gaming Profiles**: Pre-configured optimization profiles
- **Performance Tuning**: Hardware-specific optimizations

### Security
- **Content Signing**: Cosign, Notary v2, and custom signatures
- **Multi-factor Auth**: OAuth2, OIDC, basic authentication
- **TLS 1.3**: Built-in encryption for all communication

### Enterprise Features
- **Multi-backend Storage**: Filesystem, S3, GhostBay support
- **Garbage Collection**: Automated cleanup and maintenance
- **High Availability**: Clustering and failover ready
- **Audit Logging**: Complete audit trail (coming soon)

## 📖 Quick Links

- **Getting Started**: [Installation Guide](./installation.md)
- **SSO Setup**: [Azure AD](./sso/azure-ad.md) | [GitHub](./sso/github.md) | [Google](./sso/google.md)
- **API Docs**: [Registry API](./api/registry.md) | [Bolt API](./api/bolt.md) | [QUIC API](./api/quic.md)
- **Production**: [Deployment Guide](./deployment/production.md) | [Monitoring](./operations/monitoring.md)

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/CK-Technology/drift/issues)
- **Discussions**: [GitHub Discussions](https://github.com/CK-Technology/drift/discussions)
- **Security**: [Security Policy](../SECURITY.md)

## 📄 License

Drift Registry is released under the [MIT License](../LICENSE).

---

**Note**: This is a comprehensive container registry solution. For production deployments, please review the [Security Best Practices](./security/best-practices.md) and [Production Deployment Guide](./deployment/production.md).