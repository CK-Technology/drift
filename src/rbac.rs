use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::RbacConfig;

/// Organization-level Role-Based Access Control (RBAC) system
#[derive(Clone)]
pub struct RbacService {
    config: RbacConfig,
    organizations: Arc<RwLock<HashMap<String, Organization>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
    roles: Arc<RwLock<HashMap<String, Role>>>,
    permissions: Arc<RwLock<HashMap<String, Permission>>>,
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Organization entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub members: HashSet<String>, // User IDs
    pub teams: HashMap<String, Team>,
    pub repositories: HashSet<String>,
    pub settings: OrganizationSettings,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Team within an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: String,
    pub organization_id: String,
    pub members: HashSet<String>, // User IDs
    pub roles: HashSet<String>, // Role IDs
    pub repositories: HashSet<String>, // Repository access
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub organizations: HashSet<String>, // Organization IDs
    pub teams: HashSet<String>, // Team IDs
    pub direct_roles: HashSet<String>, // Direct role assignments
    pub attributes: HashMap<String, String>, // Custom attributes
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub active: bool,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>, // Permission IDs
    pub parent_role: Option<String>, // For role inheritance
    pub scope: RoleScope,
    pub priority: i32, // Higher priority overrides lower
    pub system_role: bool, // Built-in system roles
}

/// Scope of a role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoleScope {
    Global,                    // System-wide
    Organization(String),      // Organization-specific
    Repository(String),        // Repository-specific
    Namespace(String),        // Namespace-specific
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub resource: ResourceType,
    pub action: Action,
    pub conditions: Vec<Condition>,
}

/// Resource types that can be protected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Registry,
    Organization,
    Team,
    Repository,
    Image,
    Tag,
    Blob,
    Manifest,
    Signature,
    Profile,      // Bolt profiles
    Plugin,       // Bolt plugins
    User,
    Role,
    Settings,
}

/// Actions that can be performed on resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    // Read operations
    Read,
    List,
    Search,

    // Write operations
    Create,
    Update,
    Delete,

    // Repository operations
    Pull,
    Push,
    Tag,
    Sign,

    // Administrative operations
    Admin,
    ManageMembers,
    ManageRoles,
    ManageSettings,

    // Special operations
    Execute,    // For plugins
    Optimize,   // For image optimization
    Audit,      // For audit logs
}

/// Conditions for permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub type_: ConditionType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    TimeRange,      // Time-based access
    IpRange,        // IP-based restrictions
    Tag,            // Tag-based conditions
    Attribute,      // User attribute conditions
    Repository,     // Repository patterns
    Namespace,      // Namespace patterns
}

/// Organization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub require_2fa: bool,
    pub allow_public_repos: bool,
    pub default_visibility: String,
    pub max_members: Option<usize>,
    pub max_repositories: Option<usize>,
    pub storage_quota_gb: Option<u64>,
    pub allowed_domains: Vec<String>,
    pub webhook_url: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: String,
    pub organization_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: String,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Denied,
    Failed,
}

/// Authorization request
#[derive(Debug, Clone)]
pub struct AuthzRequest {
    pub user_id: String,
    pub resource: ResourceType,
    pub resource_id: String,
    pub action: Action,
    pub context: HashMap<String, String>,
}

/// Authorization response
#[derive(Debug, Clone)]
pub struct AuthzResponse {
    pub allowed: bool,
    pub reason: String,
    pub applied_roles: Vec<String>,
    pub applied_permissions: Vec<String>,
}

impl RbacService {
    pub async fn new(config: RbacConfig) -> Result<Self> {
        info!("Initializing RBAC service");

        let service = Self {
            config,
            organizations: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize default roles and permissions
        service.initialize_defaults().await?;

        info!("RBAC service initialized successfully");
        Ok(service)
    }

    /// Initialize default system roles and permissions
    async fn initialize_defaults(&self) -> Result<()> {
        let mut roles = self.roles.write().await;
        let mut permissions = self.permissions.write().await;

        // Create default permissions
        let default_permissions = vec![
            Permission {
                id: "registry.read".to_string(),
                name: "Read Registry".to_string(),
                resource: ResourceType::Registry,
                action: Action::Read,
                conditions: vec![],
            },
            Permission {
                id: "repository.pull".to_string(),
                name: "Pull Images".to_string(),
                resource: ResourceType::Repository,
                action: Action::Pull,
                conditions: vec![],
            },
            Permission {
                id: "repository.push".to_string(),
                name: "Push Images".to_string(),
                resource: ResourceType::Repository,
                action: Action::Push,
                conditions: vec![],
            },
            Permission {
                id: "repository.admin".to_string(),
                name: "Administer Repository".to_string(),
                resource: ResourceType::Repository,
                action: Action::Admin,
                conditions: vec![],
            },
            Permission {
                id: "organization.admin".to_string(),
                name: "Administer Organization".to_string(),
                resource: ResourceType::Organization,
                action: Action::Admin,
                conditions: vec![],
            },
        ];

        for perm in default_permissions {
            permissions.insert(perm.id.clone(), perm);
        }

        // Create default roles
        let default_roles = vec![
            Role {
                id: "admin".to_string(),
                name: "Administrator".to_string(),
                description: "Full system access".to_string(),
                permissions: permissions.keys().cloned().collect(),
                parent_role: None,
                scope: RoleScope::Global,
                priority: 100,
                system_role: true,
            },
            Role {
                id: "developer".to_string(),
                name: "Developer".to_string(),
                description: "Can push and pull images".to_string(),
                permissions: vec![
                    "registry.read".to_string(),
                    "repository.pull".to_string(),
                    "repository.push".to_string(),
                ].into_iter().collect(),
                parent_role: None,
                scope: RoleScope::Global,
                priority: 50,
                system_role: true,
            },
            Role {
                id: "viewer".to_string(),
                name: "Viewer".to_string(),
                description: "Read-only access".to_string(),
                permissions: vec![
                    "registry.read".to_string(),
                    "repository.pull".to_string(),
                ].into_iter().collect(),
                parent_role: None,
                scope: RoleScope::Global,
                priority: 10,
                system_role: true,
            },
        ];

        for role in default_roles {
            roles.insert(role.id.clone(), role);
        }

        info!("Initialized {} default roles and {} permissions",
            roles.len(), permissions.len());

        Ok(())
    }

    /// Check authorization for a request
    pub async fn authorize(&self, request: AuthzRequest) -> Result<AuthzResponse> {
        debug!("Authorizing request: {:?}", request);

        // Get user
        let users = self.users.read().await;
        let user = users.get(&request.user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", request.user_id))?;

        // Collect all applicable roles
        let mut applicable_roles = Vec::new();
        let roles = self.roles.read().await;

        // Add direct roles
        for role_id in &user.direct_roles {
            if let Some(role) = roles.get(role_id) {
                applicable_roles.push(role.clone());
            }
        }

        // Add team roles
        for team_id in &user.teams {
            // In real implementation, would look up team and its roles
            debug!("Checking team roles for team: {}", team_id);
        }

        // Sort roles by priority
        applicable_roles.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Check permissions
        let permissions = self.permissions.read().await;
        let mut applied_permissions = Vec::new();
        let mut allowed = false;

        for role in &applicable_roles {
            for perm_id in &role.permissions {
                if let Some(permission) = permissions.get(perm_id) {
                    if self.check_permission(&permission, &request).await {
                        applied_permissions.push(perm_id.clone());
                        allowed = true;
                    }
                }
            }

            if allowed {
                break; // Stop at first matching role
            }
        }

        // Log the authorization decision
        self.audit_authorization(&request, &allowed).await;

        Ok(AuthzResponse {
            allowed,
            reason: if allowed {
                "Permission granted".to_string()
            } else {
                "Permission denied: insufficient privileges".to_string()
            },
            applied_roles: applicable_roles.iter().map(|r| r.id.clone()).collect(),
            applied_permissions,
        })
    }

    /// Check if a permission matches the request
    async fn check_permission(&self, permission: &Permission, request: &AuthzRequest) -> bool {
        // Check resource type matches
        if permission.resource != request.resource {
            return false;
        }

        // Check action matches
        if permission.action != request.action {
            return false;
        }

        // Check conditions
        for condition in &permission.conditions {
            if !self.evaluate_condition(condition, request).await {
                return false;
            }
        }

        true
    }

    /// Evaluate a permission condition
    async fn evaluate_condition(&self, condition: &Condition, _request: &AuthzRequest) -> bool {
        match condition.type_ {
            ConditionType::TimeRange => {
                // Check if current time is within range
                true // Simplified
            }
            ConditionType::IpRange => {
                // Check if request IP is in allowed range
                true // Simplified
            }
            ConditionType::Tag => {
                // Check tag-based conditions
                true // Simplified
            }
            ConditionType::Attribute => {
                // Check user attributes
                true // Simplified
            }
            ConditionType::Repository => {
                // Check repository pattern matching
                true // Simplified
            }
            ConditionType::Namespace => {
                // Check namespace pattern matching
                true // Simplified
            }
        }
    }

    /// Create a new organization
    pub async fn create_organization(&self, org: Organization) -> Result<()> {
        let mut organizations = self.organizations.write().await;

        if organizations.contains_key(&org.id) {
            return Err(anyhow::anyhow!("Organization already exists: {}", org.id));
        }

        organizations.insert(org.id.clone(), org.clone());

        // Audit the creation
        self.audit_log.write().await.push(AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            user_id: org.owner_id.clone(),
            organization_id: Some(org.id.clone()),
            action: "create_organization".to_string(),
            resource: "organization".to_string(),
            resource_id: org.id.clone(),
            result: AuditResult::Success,
            ip_address: None,
            user_agent: None,
            details: HashMap::new(),
        });

        info!("Created organization: {}", org.id);
        Ok(())
    }

    /// Add user to organization
    pub async fn add_user_to_organization(&self, org_id: &str, user_id: &str) -> Result<()> {
        let mut organizations = self.organizations.write().await;
        let mut users = self.users.write().await;

        let org = organizations.get_mut(org_id)
            .ok_or_else(|| anyhow::anyhow!("Organization not found: {}", org_id))?;

        let user = users.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", user_id))?;

        org.members.insert(user_id.to_string());
        user.organizations.insert(org_id.to_string());

        info!("Added user {} to organization {}", user_id, org_id);
        Ok(())
    }

    /// Create a new team
    pub async fn create_team(&self, team: Team) -> Result<()> {
        let mut organizations = self.organizations.write().await;

        let org = organizations.get_mut(&team.organization_id)
            .ok_or_else(|| anyhow::anyhow!("Organization not found: {}", team.organization_id))?;

        org.teams.insert(team.id.clone(), team.clone());

        info!("Created team {} in organization {}", team.id, team.organization_id);
        Ok(())
    }

    /// Assign role to user
    pub async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        let roles = self.roles.read().await;

        if !roles.contains_key(role_id) {
            return Err(anyhow::anyhow!("Role not found: {}", role_id));
        }

        let user = users.get_mut(user_id)
            .ok_or_else(|| anyhow::anyhow!("User not found: {}", user_id))?;

        user.direct_roles.insert(role_id.to_string());

        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Create custom role
    pub async fn create_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;

        if roles.contains_key(&role.id) {
            return Err(anyhow::anyhow!("Role already exists: {}", role.id));
        }

        roles.insert(role.id.clone(), role.clone());

        info!("Created role: {}", role.id);
        Ok(())
    }

    /// Audit authorization decision
    async fn audit_authorization(&self, request: &AuthzRequest, allowed: &bool) {
        let entry = AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            user_id: request.user_id.clone(),
            organization_id: None,
            action: format!("{:?}", request.action),
            resource: format!("{:?}", request.resource),
            resource_id: request.resource_id.clone(),
            result: if *allowed { AuditResult::Success } else { AuditResult::Denied },
            ip_address: request.context.get("ip").cloned(),
            user_agent: request.context.get("user_agent").cloned(),
            details: request.context.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect(),
        };

        self.audit_log.write().await.push(entry);
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self, limit: usize) -> Vec<AuditEntry> {
        let log = self.audit_log.read().await;
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Get organization by ID
    pub async fn get_organization(&self, org_id: &str) -> Option<Organization> {
        self.organizations.read().await.get(org_id).cloned()
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.read().await.get(user_id).cloned()
    }

    /// List all roles
    pub async fn list_roles(&self) -> Vec<Role> {
        self.roles.read().await.values().cloned().collect()
    }

    /// List all permissions
    pub async fn list_permissions(&self) -> Vec<Permission> {
        self.permissions.read().await.values().cloned().collect()
    }
}