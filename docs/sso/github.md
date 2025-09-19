# GitHub OAuth SSO Configuration

This guide covers setting up GitHub OAuth as your SSO provider for Drift Registry, perfect for development teams and organizations using GitHub.

## 📋 Prerequisites

- GitHub account with admin access to an organization (for organization-level apps)
- Drift Registry deployed and accessible
- GitHub CLI installed (optional)

## 🚀 Step 1: Create GitHub OAuth App

### Via GitHub Web Interface

1. **Navigate to GitHub Settings**
   - For **Personal**: Go to Settings → Developer settings → OAuth Apps
   - For **Organization**: Go to Organization Settings → Developer settings → OAuth Apps

2. **Create New OAuth App**
   - Click **New OAuth App**
   - Fill in the details:
     ```
     Application name: Drift Container Registry
     Homepage URL: https://your-registry.com
     Application description: Container registry with GitHub authentication
     Authorization callback URL: https://your-registry.com/auth/callback/github
     ```

3. **Note Application Details**
   - Copy **Client ID**
   - Generate and copy **Client Secret**

### Via GitHub CLI

```bash
# Create OAuth app for organization
gh api -X POST /orgs/{ORG}/oauth_applications \
  -f name="Drift Container Registry" \
  -f url="https://your-registry.com" \
  -f callback_url="https://your-registry.com/auth/callback/github"

# Create OAuth app for personal account
gh api -X POST /user/oauth_applications \
  -f name="Drift Container Registry" \
  -f url="https://your-registry.com" \
  -f callback_url="https://your-registry.com/auth/callback/github"
```

## 🔧 Step 2: Configure Drift Registry

### Configuration File (drift.toml)

```toml
[auth]
mode = "oidc"
jwt_secret = "your-strong-jwt-secret-256-bits"
token_expiry_hours = 8

[auth.oauth]
enabled = true

[auth.oauth.github]
client_id = "{CLIENT_ID}"
client_secret = "{CLIENT_SECRET}"
enabled = true

# Organization restrictions (optional)
allowed_organizations = ["your-org", "another-org"]

# Team-based access control (optional)
allowed_teams = [
  "your-org/developers",
  "your-org/devops",
  "another-org/engineering"
]

# User restrictions (optional)
allowed_users = ["specific-user1", "specific-user2"]

# Scope configuration
scopes = ["user:email", "read:org"]  # Default scopes
```

### Environment Variables

```bash
# Set environment variables
export DRIFT_AUTH_MODE="oidc"
export DRIFT_AUTH_JWT_SECRET="your-256-bit-secret"
export DRIFT_GITHUB_CLIENT_ID="your-github-client-id"
export DRIFT_GITHUB_CLIENT_SECRET="your-github-client-secret"

# Optional: organization restrictions
export DRIFT_GITHUB_ALLOWED_ORGS="your-org,another-org"
export DRIFT_GITHUB_ALLOWED_TEAMS="your-org/developers,your-org/devops"

# Start Drift with environment config
drift server --config-env
```

### Docker Environment

```yaml
# docker-compose.yml
version: '3.8'
services:
  drift-registry:
    image: drift:latest
    environment:
      - DRIFT_AUTH_MODE=oidc
      - DRIFT_AUTH_JWT_SECRET=${JWT_SECRET}
      - DRIFT_GITHUB_CLIENT_ID=${GITHUB_CLIENT_ID}
      - DRIFT_GITHUB_CLIENT_SECRET=${GITHUB_CLIENT_SECRET}
      - DRIFT_GITHUB_ALLOWED_ORGS=${GITHUB_ALLOWED_ORGS}
    ports:
      - "5000:5000"
    volumes:
      - ./data:/app/data
```

## 🧪 Step 3: Test Authentication

### Web Browser Test

1. **Navigate to login page**:
   ```
   https://your-registry.com/auth/login/github
   ```

2. **Expected flow**:
   - Redirects to GitHub OAuth
   - User authorizes application
   - Redirects back to registry
   - User is logged in

### API Test

```bash
# Get authorization URL
curl -X GET "https://your-registry.com/auth/login/github"

# After web flow completion, test with token
TOKEN="your-jwt-token"

# Test authenticated registry access
curl -H "Authorization: Bearer $TOKEN" \
     -X GET "https://your-registry.com/v2/_catalog"

# Test user info endpoint
curl -H "Authorization: Bearer $TOKEN" \
     -X GET "https://your-registry.com/auth/user"
```

### Docker Client Test

```bash
# Login via Docker client
docker login your-registry.com
# Username: github-username
# Password: personal-access-token or JWT from web login

# Test operations
docker tag hello-world your-registry.com/test/hello-world
docker push your-registry.com/test/hello-world
docker pull your-registry.com/test/hello-world
```

## 🛡️ Step 4: Access Control Configuration

### Organization-Based Access

```toml
[auth.oauth.github]
# Only allow users from specific organizations
allowed_organizations = ["my-company", "partner-org"]

# Require organization membership verification
require_org_membership = true

# Allow public repositories only
allow_public_repos_only = false
```

### Team-Based Access Control

```toml
[auth.oauth.github]
# Map GitHub teams to registry roles
team_mappings = [
  { team = "my-company/admins", role = "admin" },
  { team = "my-company/developers", role = "read-write" },
  { team = "my-company/ci-bots", role = "push-only" },
  { team = "partner-org/external", role = "read-only" }
]

# Require team membership
require_team_membership = true

# Teams that can access the registry
allowed_teams = [
  "my-company/backend-team",
  "my-company/frontend-team",
  "my-company/devops"
]
```

### Repository-Based Access

```toml
[auth.oauth.github]
# Grant access based on repository permissions
repository_access = true

# Map repository permissions to registry roles
repo_role_mappings = [
  { permission = "admin", role = "admin" },
  { permission = "write", role = "read-write" },
  { permission = "read", role = "read-only" }
]

# Specific repositories for access validation
allowed_repositories = [
  "my-company/api-service",
  "my-company/web-frontend",
  "my-company/infrastructure"
]
```

## 🔄 Step 5: Advanced Configuration

### Custom Scopes

```toml
[auth.oauth.github]
# Custom OAuth scopes for additional permissions
scopes = [
  "user:email",        # Read user email (required)
  "read:org",          # Read organization membership
  "read:user",         # Read user profile information
  "repo:status",       # Read repository status
  "public_repo"        # Access public repositories
]

# Enterprise-specific scopes
enterprise_scopes = [
  "read:enterprise",   # Read enterprise information
  "read:audit_log"     # Read audit logs (enterprise)
]
```

### Webhook Integration

```toml
[auth.oauth.github]
# Webhook for real-time updates
webhook_secret = "your-webhook-secret"
webhook_events = [
  "organization",      # Organization membership changes
  "team",             # Team membership changes
  "repository"        # Repository permission changes
]

# Webhook endpoint
webhook_url = "https://your-registry.com/webhooks/github"
```

### Rate Limiting & Caching

```toml
[auth.oauth.github]
# Cache GitHub API responses
cache_duration_minutes = 15
cache_user_info = true
cache_org_membership = true
cache_team_membership = true

# GitHub API rate limiting
api_rate_limit_per_hour = 5000
enable_conditional_requests = true

# Pagination settings
max_teams_per_user = 100
max_orgs_per_user = 50
```

## 🧪 Testing & Validation

### Comprehensive Test Suite

```bash
#!/bin/bash
# Test script for GitHub OAuth

# Test 1: Public access (should fail)
echo "Testing public access..."
curl -f https://your-registry.com/v2/_catalog || echo "✓ Public access blocked"

# Test 2: GitHub OAuth flow
echo "Testing GitHub OAuth..."
AUTH_URL=$(curl -s https://your-registry.com/auth/login/github | grep -o 'https://github.com/login/oauth/authorize[^"]*')
echo "Visit: $AUTH_URL"

# Test 3: Token validation
echo "Testing token validation..."
read -p "Enter JWT token: " TOKEN
curl -H "Authorization: Bearer $TOKEN" https://your-registry.com/auth/user

# Test 4: Registry operations
echo "Testing registry operations..."
curl -H "Authorization: Bearer $TOKEN" https://your-registry.com/v2/_catalog
```

### Organization Access Test

```bash
# Test organization membership
curl -H "Authorization: Bearer $TOKEN" \
     https://your-registry.com/auth/user/organizations

# Test team membership
curl -H "Authorization: Bearer $TOKEN" \
     https://your-registry.com/auth/user/teams

# Test repository access
curl -H "Authorization: Bearer $TOKEN" \
     https://your-registry.com/auth/user/repositories
```

## 🚨 Troubleshooting

### Common Issues

**"Application not found"**
```bash
# Check OAuth app exists in correct organization/account
# Verify client ID is correct
# Ensure app is not suspended
```

**"Invalid redirect URI"**
```bash
# Verify callback URL exactly matches:
https://your-registry.com/auth/callback/github
# Check for typos, trailing slashes, http vs https
```

**"Access denied - user not in organization"**
```bash
# Check user is member of allowed organization
# Verify organization privacy settings
# Ensure user has made membership public (if required)
```

**"Insufficient permissions"**
```bash
# Check OAuth app has required scopes
# Verify user granted all requested permissions
# Check team/repository access requirements
```

### Debug Configuration

```toml
[logging]
level = "debug"
auth_debug = true
github_api_debug = true

[auth.oauth.github]
# Enable detailed logging
log_api_requests = true
log_auth_decisions = true
log_membership_checks = true
```

### GitHub API Issues

```bash
# Check GitHub API status
curl https://status.github.com/api/status.json

# Test GitHub API access
curl -H "Authorization: token YOUR_PERSONAL_TOKEN" \
     https://api.github.com/user

# Check rate limits
curl -H "Authorization: token YOUR_PERSONAL_TOKEN" \
     https://api.github.com/rate_limit
```

## 📊 Monitoring & Analytics

### Authentication Metrics

```toml
[metrics]
github_auth_metrics = true
track_login_attempts = true
track_organization_access = true
track_team_access = true

# Metrics endpoints
github_api_calls_total = true
github_auth_success_total = true
github_auth_failure_total = true
```

### Key Metrics to Monitor

- **Authentication success/failure rates**
- **GitHub API rate limit usage**
- **Organization membership changes**
- **Team membership changes**
- **Token refresh rates**

### Alerting

```yaml
# Prometheus alerting rules
groups:
  - name: github-auth
    rules:
      - alert: GitHubAuthFailures
        expr: rate(github_auth_failure_total[5m]) > 0.1
        annotations:
          summary: High GitHub authentication failure rate

      - alert: GitHubRateLimitHit
        expr: github_api_rate_limit_remaining < 100
        annotations:
          summary: GitHub API rate limit nearly exhausted
```

## 🔄 Maintenance

### Regular Tasks

1. **Review OAuth app permissions** (monthly)
2. **Update allowed organizations/teams** (as needed)
3. **Rotate client secrets** (annually)
4. **Audit user access** (quarterly)

### Client Secret Rotation

```bash
# Generate new client secret in GitHub
# Update Drift configuration
# Test new secret works
# Remove old secret reference
```

### Organization/Team Updates

```bash
# Update allowed organizations
export DRIFT_GITHUB_ALLOWED_ORGS="org1,org2,new-org"

# Update allowed teams
export DRIFT_GITHUB_ALLOWED_TEAMS="org1/team1,org1/team2,new-org/team1"

# Restart Drift Registry to apply changes
systemctl restart drift-registry
```

## 🔗 Integration Examples

### CI/CD Pipeline Integration

```yaml
# GitHub Actions example
name: Deploy to Registry
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Login to Drift Registry
        uses: docker/login-action@v2
        with:
          registry: your-registry.com
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v3
        with:
          push: true
          tags: your-registry.com/${{ github.repository }}:${{ github.sha }}
```

### Personal Access Token Integration

```bash
# Create GitHub PAT with required scopes
# Use PAT for Docker login
docker login your-registry.com -u github-username -p github-personal-access-token

# Or use in CI/CD
echo "$GITHUB_PAT" | docker login your-registry.com -u github-username --password-stdin
```

## 📞 Support

### GitHub Resources
- [GitHub OAuth Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps)
- [GitHub API Documentation](https://docs.github.com/en/rest)
- [GitHub Scopes Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps/scopes-for-oauth-apps)

### Drift Registry
- [Authentication Troubleshooting](../operations/troubleshooting.md#github-auth)
- [GitHub Issues](https://github.com/CK-Technology/drift/issues)

---

**Next Steps**: [Azure AD SSO](./azure-ad.md) | [Google SSO](./google.md) | [RBAC Configuration](../auth/rbac.md)