use crate::application::harbor::types::{
    ConflictPolicy, ExportPolicy, HarborImportResourceResult, HarborResourceBundle, HarborScope,
};
use crate::error::Result;
use crate::ports::transaction_manager::Transaction;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait HarborProvider: Send + Sync {
    fn key(&self) -> &'static str;

    fn validate(&self, _resource: &HarborResourceBundle) -> Result<()> {
        Ok(())
    }

    async fn export(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        policy: ExportPolicy,
    ) -> Result<HarborResourceBundle>;

    async fn import(
        &self,
        realm_id: Uuid,
        scope: &HarborScope,
        resource: &HarborResourceBundle,
        conflict_policy: ConflictPolicy,
        dry_run: bool,
        tx: Option<&mut dyn Transaction>,
    ) -> Result<HarborImportResourceResult>;
}

pub struct HarborRegistry {
    providers: HashMap<String, Arc<dyn HarborProvider>>,
}

impl HarborRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Arc<dyn HarborProvider>) {
        self.providers.insert(provider.key().to_string(), provider);
    }

    pub fn get(&self, key: &str) -> Option<Arc<dyn HarborProvider>> {
        self.providers.get(key).cloned()
    }

    pub fn all(&self) -> Vec<Arc<dyn HarborProvider>> {
        self.providers.values().cloned().collect()
    }
}

impl Default for HarborRegistry {
    fn default() -> Self {
        Self::new()
    }
}
