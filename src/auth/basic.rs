use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub enabled: bool,
    pub realm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

pub struct BasicAuthService {
    config: BasicAuthConfig,
    users: Vec<User>,
}

impl BasicAuthService {
    pub fn new(config: BasicAuthConfig) -> Self {
        Self {
            config,
            users: Vec::new(),
        }
    }

    pub fn add_user(&mut self, username: String, password: String, email: Option<String>, roles: Vec<String>) -> Result<()> {
        let password_hash = hash(password, DEFAULT_COST)?;
        let user = User {
            username,
            password_hash,
            email,
            roles,
        };
        self.users.push(user);
        Ok(())
    }

    pub fn authenticate(&self, auth_header: &str) -> Result<Option<User>> {
        if !self.config.enabled {
            return Ok(None);
        }

        if !auth_header.starts_with("Basic ") {
            return Ok(None);
        }

        let encoded = auth_header.strip_prefix("Basic ").unwrap();
        let decoded = general_purpose::STANDARD.decode(encoded)?;
        let credentials = String::from_utf8(decoded)?;

        let mut parts = credentials.splitn(2, ':');
        let username = parts.next().ok_or_else(|| anyhow::anyhow!("Invalid credentials format"))?;
        let password = parts.next().ok_or_else(|| anyhow::anyhow!("Invalid credentials format"))?;

        for user in &self.users {
            if user.username == username && verify(password, &user.password_hash)? {
                return Ok(Some(user.clone()));
            }
        }

        Ok(None)
    }
}