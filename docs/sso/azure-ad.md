# Azure Active Directory (Azure AD) SSO Configuration

This guide walks you through setting up Microsoft Azure Active Directory as your SSO provider for Drift Registry.

## 📋 Prerequisites

- Azure AD tenant with admin access
- Drift Registry deployed and accessible
- Azure CLI installed (optional, for automation)

## 🚀 Step 1: Create Azure AD Application

### Via Azure Portal

1. **Navigate to Azure Portal**
   - Go to [portal.azure.com](https://portal.azure.com)
   - Select **Azure Active Directory**

2. **Register Application**
   - Click **App registrations** → **New registration**
   - Fill in the details:
     ```
     Name: Drift Container Registry
     Supported account types: Accounts in this organizational directory only
     Redirect URI: Web → https://your-registry.com/auth/callback/azure
     ```

3. **Note Application Details**
   - Copy **Application (client) ID**
   - Copy **Directory (tenant) ID**

4. **Create Client Secret**
   - Go to **Certificates & secrets**
   - Click **New client secret**
   - Set description: "Drift Registry Secret"
   - Set expiry: 24 months (recommended)
   - Copy the **Value** (not the ID)

### Via Azure CLI

```bash
# Create the application
az ad app create \
  --display-name "Drift Container Registry" \
  --web-redirect-uris "https://your-registry.com/auth/callback/azure" \
  --sign-in-audience "AzureADMyOrg"

# Get the app ID
APP_ID=$(az ad app list --display-name "Drift Container Registry" --query "[0].appId" -o tsv)

# Create service principal
az ad sp create --id $APP_ID

# Create client secret
az ad app credential reset --id $APP_ID --display-name "Drift Registry Secret"
```

## ⚙️ Step 2: Configure API Permissions

### Required Permissions

1. **Microsoft Graph API**:
   - `User.Read` (Read user profile)
   - `profile` (Read user's basic profile)
   - `openid` (Sign in and read user profile)
   - `email` (Read user's email address)

### Setup via Portal

1. **Go to API permissions**
   - In your app registration, click **API permissions**
   - Click **Add a permission**

2. **Add Microsoft Graph permissions**:
   ```
   Microsoft Graph → Delegated permissions:
   ✓ User.Read
   ✓ profile
   ✓ openid
   ✓ email
   ```

3. **Grant admin consent**
   - Click **Grant admin consent for [Your Org]**
   - Confirm the consent

## 🔧 Step 3: Configure Drift Registry

### Configuration File (drift.toml)

```toml
[auth]
mode = "oidc"
jwt_secret = "your-strong-jwt-secret-256-bits"
token_expiry_hours = 8

[auth.oidc]
issuer = "https://login.microsoftonline.com/{TENANT_ID}/v2.0"
client_id = "{CLIENT_ID}"
client_secret = "{CLIENT_SECRET}"

[auth.oauth]
enabled = true

[auth.oauth.azure]
client_id = "{CLIENT_ID}"
client_secret = "{CLIENT_SECRET}"
tenant_id = "{TENANT_ID}"
enabled = true
# Optional: restrict to specific groups
allowed_groups = ["drift-users", "container-registry-users"]
```

### Environment Variables

For production, use environment variables:

```bash
# Set environment variables
export DRIFT_AUTH_MODE="oidc"
export DRIFT_AUTH_JWT_SECRET="your-256-bit-secret"
export DRIFT_AZURE_CLIENT_ID="your-client-id"
export DRIFT_AZURE_CLIENT_SECRET="your-client-secret"
export DRIFT_AZURE_TENANT_ID="your-tenant-id"

# Start Drift with environment config
drift server --config-env
```

### Docker Environment

```dockerfile
# In your docker-compose.yml
services:
  drift-registry:
    image: drift:latest
    environment:
      - DRIFT_AUTH_MODE=oidc
      - DRIFT_AUTH_JWT_SECRET=${JWT_SECRET}
      - DRIFT_AZURE_CLIENT_ID=${AZURE_CLIENT_ID}
      - DRIFT_AZURE_CLIENT_SECRET=${AZURE_CLIENT_SECRET}
      - DRIFT_AZURE_TENANT_ID=${AZURE_TENANT_ID}
    ports:
      - "5000:5000"
```

## 🧪 Step 4: Test Authentication

### Web Browser Test

1. **Navigate to login page**:
   ```
   https://your-registry.com/auth/login/azure
   ```

2. **Expected flow**:
   - Redirects to Microsoft login
   - User enters Azure AD credentials
   - Redirects back to registry
   - User is logged in

### API Test

```bash
# Start authentication flow
curl -X GET "https://your-registry.com/auth/login/azure"

# After completing web flow, test with token
TOKEN="your-jwt-token"

# Test authenticated API call
curl -H "Authorization: Bearer $TOKEN" \
     -X GET "https://your-registry.com/v2/_catalog"
```

### Docker Client Test

```bash
# Login via Docker client
docker login your-registry.com
# This should redirect to Azure AD login

# Test push/pull
docker tag hello-world your-registry.com/test/hello-world
docker push your-registry.com/test/hello-world
```

## 🛡️ Step 5: Security Hardening

### Conditional Access Policies

Create Azure AD Conditional Access policies:

```json
{
  "displayName": "Drift Registry - Require MFA",
  "state": "enabled",
  "conditions": {
    "applications": {
      "includeApplications": ["{YOUR_APP_ID}"]
    },
    "users": {
      "includeUsers": ["All"]
    }
  },
  "grantControls": {
    "builtInControls": ["mfa"],
    "operator": "OR"
  }
}
```

### App-level Security

```toml
[auth.oauth.azure]
# Require specific Azure AD groups
allowed_groups = ["container-registry-users"]

# Require specific roles
required_roles = ["Registry.Read", "Registry.Write"]

# Token validation settings
validate_issuer = true
validate_audience = true
clock_skew_seconds = 300
```

## 🔄 Advanced Configuration

### Group-based Access Control

```toml
[auth.oauth.azure]
# Map Azure AD groups to registry roles
group_mappings = [
  { group = "registry-admins", role = "admin" },
  { group = "developers", role = "read-write" },
  { group = "ci-cd", role = "push-only" }
]
```

### Custom Claims

```toml
[auth.oauth.azure]
# Use custom claims for authorization
custom_claims = [
  "department",
  "employee_type",
  "cost_center"
]

# Department-based restrictions
allowed_departments = ["Engineering", "DevOps"]
```

### Multiple Azure AD Tenants

```toml
[auth.oauth.azure_prod]
client_id = "prod-client-id"
tenant_id = "production-tenant-id"
enabled = true

[auth.oauth.azure_dev]
client_id = "dev-client-id"
tenant_id = "development-tenant-id"
enabled = true
```

## 🚨 Troubleshooting

### Common Issues

**"AADSTS50011: The reply URL specified in the request does not match"**
```bash
# Check redirect URI in Azure AD exactly matches:
https://your-registry.com/auth/callback/azure
# No trailing slash, correct protocol (https)
```

**"AADSTS70001: Application not found in directory"**
```bash
# Verify tenant ID is correct
# Check application ID is correct
# Ensure app registration exists in correct tenant
```

**"Invalid client secret"**
```bash
# Client secret may have expired
# Generate new secret in Azure portal
# Update configuration with new secret
```

**"Access denied - insufficient privileges"**
```bash
# Check user is in allowed groups
# Verify API permissions are granted
# Check admin consent was provided
```

### Debug Configuration

```toml
[logging]
level = "debug"
auth_debug = true

[auth]
# Enable detailed auth logging
log_auth_events = true
log_failed_attempts = true
```

### Validate Configuration

```bash
# Test OIDC discovery
curl https://login.microsoftonline.com/{tenant-id}/v2.0/.well-known/openid_configuration

# Validate JWT tokens
# Use jwt.io to decode and verify tokens
```

## 📊 Monitoring & Analytics

### Authentication Metrics

Monitor these Azure AD metrics:
- Sign-in frequency
- Failed sign-in attempts
- MFA completion rates
- Conditional access blocks

### Drift Registry Metrics

```toml
[metrics]
# Enable auth metrics
auth_metrics = true
failed_login_threshold = 5
```

Monitor:
- Authentication success/failure rates
- Token validation failures
- Group membership changes
- Permission escalations

## 🔄 Maintenance

### Regular Tasks

1. **Rotate client secrets** (every 6-12 months)
2. **Review group memberships** (monthly)
3. **Audit conditional access policies** (quarterly)
4. **Update API permissions** (as needed)

### Secret Rotation

```bash
# Generate new secret
az ad app credential reset --id $APP_ID --display-name "Drift Registry Secret 2024"

# Update Drift configuration
# Test new secret works
# Remove old secret
```

## 📞 Support

### Microsoft Resources
- [Azure AD Documentation](https://docs.microsoft.com/en-us/azure/active-directory/)
- [App Registration Guide](https://docs.microsoft.com/en-us/azure/active-directory/develop/quickstart-register-app)
- [OIDC with Azure AD](https://docs.microsoft.com/en-us/azure/active-directory/develop/v2-protocols-oidc)

### Drift Registry
- [Authentication Issues](../operations/troubleshooting.md#authentication)
- [GitHub Issues](https://github.com/CK-Technology/drift/issues)

---

**Next Steps**: [Configure RBAC](../auth/rbac.md) | [GitHub SSO](./github.md) | [Monitoring Setup](../operations/monitoring.md)