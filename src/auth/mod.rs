use crate::config::{AuthConfig, AuthMode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod basic;
pub mod jwt;
pub mod oidc;
pub mod oauth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub user: User,
    pub exp: u64,
    pub iat: u64,
}

pub struct AuthService {
    mode: AuthMode,
    jwt_secret: String,
    users: HashMap<String, String>, // username -> password hash
}

impl AuthService {
    pub fn new(config: &AuthConfig) -> Result<Self> {
        let mut users = HashMap::new();

        if let Some(basic_config) = &config.basic {
            for user_entry in &basic_config.users {
                if let Some((username, password)) = user_entry.split_once(':') {
                    // In production, passwords should be hashed
                    users.insert(username.to_string(), password.to_string());
                }
            }
        }

        Ok(Self {
            mode: config.mode.clone(),
            jwt_secret: config.jwt_secret.clone(),
            users,
        })
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>> {
        match self.mode {
            AuthMode::Basic => {
                if let Some(stored_password) = self.users.get(username) {
                    if stored_password == password {
                        return Ok(Some(User {
                            username: username.to_string(),
                            roles: vec!["user".to_string()],
                            scopes: vec![
                                "repository:*:pull".to_string(),
                                "repository:*:push".to_string(),
                            ],
                        }));
                    }
                }
                Ok(None)
            }
            AuthMode::Token => {
                // TODO: Implement token authentication
                todo!("Token authentication not implemented")
            }
            AuthMode::Oidc => {
                // TODO: Implement OIDC authentication
                todo!("OIDC authentication not implemented")
            }
        }
    }

    pub fn generate_token(&self, user: &User, expires_in: u64) -> Result<String> {
        jwt::generate_token(&self.jwt_secret, user, expires_in)
    }

    pub fn validate_token(&self, token: &str) -> Result<Option<User>> {
        jwt::validate_token(&self.jwt_secret, token)
    }

    pub fn check_scope(&self, user: &User, required_scope: &str) -> bool {
        // Check if user has the required scope
        for scope in &user.scopes {
            if scope == required_scope || scope == "registry:*" {
                return true;
            }

            // Check wildcard patterns
            if let Some(prefix) = scope.strip_suffix("*") {
                if required_scope.starts_with(prefix) {
                    return true;
                }
            }
        }
        false
    }
}