use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashSet;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct AuthManager {
    client_tokens: HashSet<String>,
    admin_tokens: HashSet<String>,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut auth_manager = Self {
            client_tokens: HashSet::new(),
            admin_tokens: HashSet::new(),
        };

        // Default tokens for demo purposes
        // In production, these should be loaded from secure configuration
        auth_manager.add_client_token("client_token_123").unwrap();
        auth_manager.add_admin_token("admin_token_456").unwrap();

        auth_manager
    }

    pub fn add_client_token(&mut self, token: &str) -> Result<()> {
        let hashed = hash(token, DEFAULT_COST)?;
        self.client_tokens.insert(hashed);
        debug!("Added client token");
        Ok(())
    }

    pub fn add_admin_token(&mut self, token: &str) -> Result<()> {
        let hashed = hash(token, DEFAULT_COST)?;
        self.admin_tokens.insert(hashed);
        debug!("Added admin token");
        Ok(())
    }

    pub fn verify_client_token(&self, token: &str) -> bool {
        for hashed_token in &self.client_tokens {
            if verify(token, hashed_token).unwrap_or(false) {
                debug!("Client token verified successfully");
                return true;
            }
        }
        warn!("Client token verification failed");
        false
    }

    pub fn verify_admin_token(&self, token: &str) -> bool {
        for hashed_token in &self.admin_tokens {
            if verify(token, hashed_token).unwrap_or(false) {
                debug!("Admin token verified successfully");
                return true;
            }
        }
        warn!("Admin token verification failed");
        false
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let mut auth_manager = AuthManager::new();

        // Test adding and verifying client token
        auth_manager.add_client_token("test_client_token").unwrap();
        assert!(auth_manager.verify_client_token("test_client_token"));
        assert!(!auth_manager.verify_client_token("wrong_token"));

        // Test adding and verifying admin token
        auth_manager.add_admin_token("test_admin_token").unwrap();
        assert!(auth_manager.verify_admin_token("test_admin_token"));
        assert!(!auth_manager.verify_admin_token("wrong_token"));
    }

    #[test]
    fn test_default_tokens() {
        let auth_manager = AuthManager::new();
        assert!(auth_manager.verify_client_token("client_token_123"));
        assert!(auth_manager.verify_admin_token("admin_token_456"));
    }
}
