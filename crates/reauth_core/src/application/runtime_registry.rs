use crate::ports::authenticator::Authenticator;
use std::collections::HashMap;
use std::sync::Arc;

/// This maps the "Node ID" (string) to the actual Rust implementation (code).
pub struct RuntimeRegistry {
    authenticators: HashMap<String, Arc<dyn Authenticator>>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self {
            authenticators: HashMap::new(),
        }
    }

    /// Register a worker (e.g., PasswordAuthenticator) under a specific key (e.g., "core.auth.password")
    pub fn register(&mut self, key: &str, implementation: Arc<dyn Authenticator>) {
        self.authenticators.insert(key.to_string(), implementation);
    }

    pub fn get_authenticator(&self, key: &str) -> Option<Arc<dyn Authenticator>> {
        self.authenticators.get(key).cloned()
    }
}
