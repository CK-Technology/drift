# Role-Based Access Control (RBAC)

Drift Registry implements a comprehensive organization-level RBAC system that provides fine-grained access control for users, teams, and resources across multiple organizations.

## 🏗️ Architecture Overview

### Core Components

- **Organizations**: Top-level isolation boundaries
- **Teams**: Groups of users within organizations
- **Users**: Individual accounts with roles and permissions
- **Roles**: Collections of permissions
- **Permissions**: Specific actions on resources
- **Resources**: Registry objects (repositories, images, etc.)

### Hierarchy

```
Organizations
├── Teams
│   ├── Members (Users)
│   └── Roles
├── Repositories
├── Settings
└── Direct User Assignments
```

## 🔧 Configuration

### Basic RBAC Setup

```toml
[rbac]
enabled = true
default_role = "viewer"
enable_organization_isolation = true
enable_team_based_access = true
enable_attribute_based_access = false
cache_ttl_seconds = 300
audit_authorization_decisions = true
```

### Environment Variables

```bash
export DRIFT_RBAC_ENABLED=true
export DRIFT_RBAC_DEFAULT_ROLE=viewer
export DRIFT_RBAC_ORG_ISOLATION=true
```

## 👥 Organizations

### Creating Organizations

```bash
# Via API
curl -X POST https://registry.example.com/api/organizations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "acme-corp",
    "name": "ACME Corporation",
    "description": "ACME development team",
    "owner_id": "john.doe",
    "settings": {
      "require_2fa": true,
      "allow_public_repos": false,
      "default_visibility": "private",
      "max_members": 100,
      "storage_quota_gb": 1000
    }
  }'
```

### Organization Settings

```toml
[organizations.acme-corp]
require_2fa = true
allow_public_repos = false
default_visibility = "private"
max_members = 100
max_repositories = 500
storage_quota_gb = 1000
allowed_domains = ["acme.com", "acme.co.uk"]
webhook_url = "https://acme.com/webhooks/registry"
```

## 👨‍👩‍👧‍👦 Teams

### Team Structure

```json
{
  "id": "backend-team",
  "name": "Backend Developers",
  "description": "Server-side development team",
  "organization_id": "acme-corp",
  "members": ["john.doe", "jane.smith", "bob.wilson"],
  "roles": ["developer", "acme-corp.backend-lead"],
  "repositories": ["acme-corp/*-api", "acme-corp/*-service"]
}
```

### Creating Teams

```bash
curl -X POST https://registry.example.com/api/organizations/acme-corp/teams \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "frontend-team",
    "name": "Frontend Developers",
    "description": "UI/UX development team",
    "repositories": ["acme-corp/*-web", "acme-corp/*-mobile"]
  }'
```

## 🎭 Roles and Permissions

### Built-in Roles

#### Global Roles

```yaml
admin:
  description: "Full system access"
  permissions: ["*"]
  scope: global

developer:
  description: "Can push and pull images"
  permissions:
    - "registry.read"
    - "repository.pull"
    - "repository.push"
    - "image.tag"
  scope: global

viewer:
  description: "Read-only access"
  permissions:
    - "registry.read"
    - "repository.pull"
  scope: global
```

#### Organization Roles

```yaml
org-admin:
  description: "Organization administrator"
  permissions:
    - "organization.admin"
    - "team.manage"
    - "repository.*"
  scope: organization

team-lead:
  description: "Team leader"
  permissions:
    - "team.manage_members"
    - "repository.admin"
  scope: team

contributor:
  description: "Team contributor"
  permissions:
    - "repository.pull"
    - "repository.push"
  scope: repository
```

### Custom Roles

```bash
# Create custom role
curl -X POST https://registry.example.com/api/roles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "id": "ci-bot",
    "name": "CI/CD Bot",
    "description": "Automated CI/CD operations",
    "permissions": [
      "repository.pull",
      "repository.push",
      "image.sign"
    ],
    "scope": "repository",
    "priority": 60
  }'
```

### Permission Types

#### Resource Types

```yaml
Registry:       # Global registry access
Organization:   # Organization-level access
Team:          # Team management
Repository:    # Repository access
Image:         # Image operations
Tag:           # Tag operations
Blob:          # Blob operations
Manifest:      # Manifest operations
Signature:     # Content signing
Profile:       # Bolt profiles
Plugin:        # Bolt plugins
User:          # User management
Role:          # Role management
Settings:      # Configuration management
```

#### Actions

```yaml
Read:           # View resources
List:           # List collections
Search:         # Search operations
Create:         # Create new resources
Update:         # Modify existing resources
Delete:         # Remove resources
Pull:           # Download images
Push:           # Upload images
Tag:            # Create/modify tags
Sign:           # Sign content
Admin:          # Administrative access
ManageMembers:  # Add/remove team members
ManageRoles:    # Assign/revoke roles
ManageSettings: # Modify settings
Execute:        # Run plugins
Optimize:       # Image optimization
Audit:          # Access audit logs
```

### Conditional Permissions

```yaml
time-restricted:
  type: "time_range"
  value: "09:00-17:00"
  description: "Business hours only"

ip-restricted:
  type: "ip_range"
  value: "192.168.1.0/24"
  description: "Internal network only"

tag-restricted:
  type: "tag"
  value: "!latest"
  description: "Cannot modify latest tag"

repo-pattern:
  type: "repository"
  value: "acme-corp/web-*"
  description: "Web repositories only"
```

## 👤 User Management

### User Structure

```json
{
  "id": "john.doe",
  "username": "johndoe",
  "email": "john.doe@acme.com",
  "full_name": "John Doe",
  "organizations": ["acme-corp", "opensource-org"],
  "teams": ["acme-corp/backend-team", "acme-corp/security-team"],
  "direct_roles": ["developer"],
  "attributes": {
    "department": "Engineering",
    "level": "Senior",
    "clearance": "Standard"
  },
  "active": true
}
```

### Role Assignment

```bash
# Assign role to user
curl -X POST https://registry.example.com/api/users/john.doe/roles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_id": "developer",
    "scope": "global"
  }'

# Assign organization-specific role
curl -X POST https://registry.example.com/api/users/john.doe/roles \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role_id": "org-admin",
    "scope": "organization",
    "scope_id": "acme-corp"
  }'
```

## 🔐 Authorization Flow

### Request Authorization

```rust
use drift::rbac::{AuthzRequest, ResourceType, Action};

// Create authorization request
let request = AuthzRequest {
    user_id: "john.doe".to_string(),
    resource: ResourceType::Repository,
    resource_id: "acme-corp/web-frontend".to_string(),
    action: Action::Push,
    context: hashmap!{
        "ip".to_string() => "192.168.1.100".to_string(),
        "user_agent".to_string() => "docker/20.10.0".to_string(),
    },
};

// Check authorization
let response = rbac_service.authorize(request).await?;

if response.allowed {
    // Proceed with operation
    println!("Access granted: {}", response.reason);
} else {
    // Deny access
    println!("Access denied: {}", response.reason);
}
```

### API Integration

```bash
# All API calls include authorization check
curl -X GET https://registry.example.com/v2/acme-corp/web-frontend/tags/list \
  -H "Authorization: Bearer $TOKEN"

# Returns 200 if authorized, 403 if denied
```

## 🏢 Multi-tenancy

### Organization Isolation

```toml
[rbac]
enable_organization_isolation = true

# Users can only see their organization's resources
# Cross-organization access requires explicit permissions
```

### Repository Namespacing

```bash
# Organization-scoped repositories
acme-corp/web-frontend
acme-corp/api-backend
acme-corp/mobile-app

# Personal repositories (if allowed)
john.doe/personal-project

# Global repositories (if allowed)
public/base-images
```

### Cross-Organization Access

```yaml
# Special role for cross-org access
cross-org-viewer:
  description: "Can view resources across organizations"
  permissions:
    - "registry.read"
    - "organization.list"
    - "repository.pull"
  scope: global
  conditions:
    - type: "attribute"
      value: "level=Executive"
```

## 📊 Audit and Monitoring

### Authorization Auditing

```json
{
  "id": "auth-123456",
  "timestamp": "2024-01-15T10:30:00Z",
  "user_id": "john.doe",
  "organization_id": "acme-corp",
  "action": "Push",
  "resource": "Repository",
  "resource_id": "acme-corp/web-frontend",
  "result": "Success",
  "applied_roles": ["developer"],
  "applied_permissions": ["repository.push"],
  "ip_address": "192.168.1.100"
}
```

### RBAC Metrics

```yaml
# Prometheus metrics
drift_rbac_authorization_total{result="allowed"}
drift_rbac_authorization_total{result="denied"}
drift_rbac_permission_checks_total
drift_rbac_role_cache_hits_total
drift_rbac_role_cache_misses_total
```

### Monitoring Dashboard

```json
{
  "dashboard": "RBAC Overview",
  "panels": [
    {
      "title": "Authorization Decisions",
      "query": "rate(drift_rbac_authorization_total[5m])"
    },
    {
      "title": "Access Denied Events",
      "query": "drift_rbac_authorization_total{result=\"denied\"}"
    },
    {
      "title": "Top Users by Activity",
      "query": "topk(10, sum by (user_id) (drift_rbac_authorization_total))"
    }
  ]
}
```

## 🔧 Advanced Features

### Attribute-Based Access Control (ABAC)

```yaml
abac-rule:
  name: "Department-based access"
  condition: |
    user.department == "Engineering" AND
    resource.namespace == user.department.toLowerCase() AND
    time.hour >= 9 AND time.hour <= 17
  effect: "allow"
```

### Dynamic Role Assignment

```yaml
auto-roles:
  - trigger: "user.joins_team"
    condition: "team.name == 'security-team'"
    action: "assign_role"
    role: "security-analyst"

  - trigger: "user.attribute_change"
    condition: "user.level == 'Manager'"
    action: "assign_role"
    role: "team-lead"
```

### Role Inheritance

```yaml
senior-developer:
  parent_role: "developer"
  additional_permissions:
    - "repository.admin"
    - "image.sign"
  description: "Developer with additional privileges"
```

## 🛡️ Security Best Practices

### Principle of Least Privilege

```yaml
# Start with minimal permissions
new-user:
  default_role: "viewer"
  permissions: ["registry.read"]

# Grant additional permissions as needed
developer-onboarding:
  week_1: ["viewer"]
  week_2: ["viewer", "limited-push"]
  week_4: ["developer"]
```

### Regular Access Reviews

```bash
# Generate access report
curl -X GET https://registry.example.com/api/reports/access \
  -H "Authorization: Bearer $TOKEN" \
  > access-report.json

# Review unused permissions
curl -X GET https://registry.example.com/api/reports/unused-permissions \
  -H "Authorization: Bearer $TOKEN"
```

### Sensitive Operations

```yaml
# Require additional confirmation for sensitive operations
sensitive-actions:
  - "organization.delete"
  - "repository.delete"
  - "user.delete"
  - "role.admin"

confirmation-required:
  method: "email"
  timeout_minutes: 5
  require_reason: true
```

## 🔄 Migration and Import

### From Basic Auth

```bash
# Export existing users
curl -X GET https://registry.example.com/api/users/export \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  > users.json

# Import with RBAC mapping
curl -X POST https://registry.example.com/api/rbac/import \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d @users.json
```

### Role Mapping

```yaml
# Map existing users to roles
role-mapping:
  - username_pattern: "*-admin"
    role: "admin"
  - email_domain: "acme.com"
    role: "developer"
  - attribute:
      department: "QA"
    role: "tester"
```

## 📞 API Reference

### Organizations API

```bash
# List organizations
GET /api/organizations

# Get organization
GET /api/organizations/{org_id}

# Create organization
POST /api/organizations

# Update organization
PUT /api/organizations/{org_id}

# Delete organization
DELETE /api/organizations/{org_id}

# Add member
POST /api/organizations/{org_id}/members

# Remove member
DELETE /api/organizations/{org_id}/members/{user_id}
```

### Teams API

```bash
# List teams
GET /api/organizations/{org_id}/teams

# Create team
POST /api/organizations/{org_id}/teams

# Add team member
POST /api/organizations/{org_id}/teams/{team_id}/members

# Assign team role
POST /api/organizations/{org_id}/teams/{team_id}/roles
```

### Roles API

```bash
# List roles
GET /api/roles

# Create role
POST /api/roles

# Assign role to user
POST /api/users/{user_id}/roles

# Check user permissions
GET /api/users/{user_id}/permissions
```

## 🚨 Troubleshooting

### Common Issues

**"Permission denied"**
```bash
# Check user roles
curl -X GET https://registry.example.com/api/users/john.doe/roles \
  -H "Authorization: Bearer $TOKEN"

# Check effective permissions
curl -X GET https://registry.example.com/api/users/john.doe/permissions \
  -H "Authorization: Bearer $TOKEN"
```

**"User not in organization"**
```bash
# Add user to organization
curl -X POST https://registry.example.com/api/organizations/acme-corp/members \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"user_id": "john.doe"}'
```

**"Role not found"**
```bash
# List available roles
curl -X GET https://registry.example.com/api/roles \
  -H "Authorization: Bearer $TOKEN"
```

### Debug Mode

```toml
[rbac]
debug_authorization = true
log_permission_checks = true
cache_debug = true
```

## 📞 Support

- [Authentication Guide](./authentication.md)
- [SSO Configuration](../sso/README.md)
- [Audit Logging](../operations/audit.md)
- [GitHub Issues](https://github.com/CK-Technology/drift/issues)

---

**Next Steps**: [Audit Logging](../operations/audit.md) | [Clustering](../deployment/clustering.md) | [Monitoring](../operations/monitoring.md)