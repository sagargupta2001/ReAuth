use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[async_trait]
pub trait LogPublisher: Send + Sync {
    async fn publish(&self, log: LogEntry);
}

pub trait LogSubscriber: Send + Sync {
    fn subscribe(&self) -> tokio::sync::broadcast::Receiver<LogEntry>;
}
