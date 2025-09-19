# Single Sign-On (SSO) Configuration

Drift Registry supports multiple SSO providers for seamless authentication integration. This section provides comprehensive guides for configuring each supported provider.

## 🔐 Supported Providers

### Microsoft Azure AD
- **Guide**: [Azure AD Setup](./azure-ad.md)
- **Features**: Enterprise SSO, conditional access, MFA
- **Best For**: Enterprise environments with Microsoft 365

### GitHub OAuth
- **Guide**: [GitHub Setup](./github.md)
- **Features**: Developer-friendly, organization access
- **Best For**: Development teams, open source projects

### Google Workspace
- **Guide**: [Google Setup](./google.md)
- **Features**: Google Workspace integration, admin controls
- **Best For**: Organizations using Google Workspace

### Custom OIDC
- **Guide**: [Custom OIDC](./custom-oidc.md)
- **Features**: Any OIDC-compliant provider
- **Best For**: Custom identity providers, Auth0, Okta

## 🚀 Quick Start

### 1. Choose Your Provider
Select the SSO provider that best fits your organization:

```toml
[auth]
mode = "oidc"  # Enable OIDC mode

[auth.oauth]
enabled = true

# Enable your preferred provider
[auth.oauth.azure]
client_id = "your-azure-client-id"
client_secret = "your-azure-client-secret"
tenant_id = "your-tenant-id"
```

### 2. Configure Provider
Follow the specific setup guide for your chosen provider:
- [Azure AD Configuration](./azure-ad.md#configuration)
- [GitHub OAuth Configuration](./github.md#configuration)
- [Google Workspace Configuration](./google.md#configuration)

### 3. Test Authentication
```bash
# Test SSO login
curl -X GET "https://your-registry.com/auth/login/azure"

# Verify user info
curl -H "Authorization: Bearer YOUR_TOKEN" \
     -X GET "https://your-registry.com/auth/user"
```

## 🔧 Configuration Reference

### Basic SSO Configuration
```toml
[auth]
mode = "oidc"
jwt_secret = "your-jwt-secret-key"
token_expiry_hours = 24

[auth.oidc]
issuer = "https://login.microsoftonline.com/{tenant-id}/v2.0"
client_id = "your-client-id"
client_secret = "your-client-secret"

[auth.oauth]
enabled = true
```

### Multi-Provider Setup
```toml
[auth.oauth.azure]
client_id = "azure-client-id"
client_secret = "azure-client-secret"
tenant_id = "your-tenant-id"
enabled = true

[auth.oauth.github]
client_id = "github-client-id"
client_secret = "github-client-secret"
enabled = true

[auth.oauth.google]
client_id = "google-client-id"
client_secret = "google-client-secret"
enabled = false  # Disabled in this example
```

## 🛡️ Security Considerations

### JWT Configuration
```toml
[auth]
jwt_secret = "use-a-strong-random-secret"  # Use 256-bit random key
token_expiry_hours = 8  # Shorter for production
```

### Redirect URLs
Ensure your OAuth applications are configured with secure redirect URLs:
- `https://your-registry.com/auth/callback/azure`
- `https://your-registry.com/auth/callback/github`
- `https://your-registry.com/auth/callback/google`

### Environment Variables
Store sensitive values in environment variables:
```bash
export DRIFT_AUTH_JWT_SECRET="your-secret-key"
export DRIFT_AZURE_CLIENT_SECRET="azure-secret"
export DRIFT_GITHUB_CLIENT_SECRET="github-secret"
```

## 📊 Provider Comparison

| Provider | Setup Complexity | Features | Enterprise | Cost |
|----------|-----------------|----------|------------|------|
| Azure AD | Medium | Advanced SSO, MFA, Conditional Access | ✅ | Paid |
| GitHub | Easy | Developer-focused, Org access | ⚠️ Limited | Free/Paid |
| Google | Easy | Workspace integration | ✅ | Paid |
| Custom OIDC | Varies | Depends on provider | ✅ | Varies |

## 🔄 Migration Guide

### From Basic Auth to SSO
1. **Backup existing users**: Export current user data
2. **Configure SSO**: Set up your chosen provider
3. **Test thoroughly**: Verify authentication works
4. **Migrate users**: Map existing users to SSO identities
5. **Update clients**: Configure Docker clients for token auth

### Between SSO Providers
1. **Configure new provider**: Add alongside existing
2. **Test new provider**: Verify it works correctly
3. **Migrate users gradually**: Allow both during transition
4. **Remove old provider**: Once migration is complete

## 🚨 Troubleshooting

### Common Issues

**"Invalid redirect URI"**
```bash
# Check your OAuth app configuration
# Ensure redirect URIs match exactly:
https://your-registry.com/auth/callback/azure
```

**"Token validation failed"**
```bash
# Check JWT secret configuration
# Verify token hasn't expired
# Ensure issuer URL is correct
```

**"Provider not found"**
```bash
# Verify provider is enabled in config
# Check client ID/secret are correct
# Ensure provider endpoints are accessible
```

### Debug Mode
Enable debug logging for authentication:
```toml
[logging]
level = "debug"
auth_debug = true
```

## 📝 Next Steps

1. **Choose your provider**: [Azure AD](./azure-ad.md) | [GitHub](./github.md) | [Google](./google.md)
2. **Configure RBAC**: [Role-Based Access Control](../auth/rbac.md)
3. **Set up monitoring**: [Authentication Metrics](../operations/monitoring.md#authentication)
4. **Security hardening**: [Best Practices](../security/best-practices.md)

## 📞 Support

For SSO configuration help:
- Check provider-specific documentation
- Review [Troubleshooting Guide](../operations/troubleshooting.md)
- Open an issue on [GitHub](https://github.com/CK-Technology/drift/issues)