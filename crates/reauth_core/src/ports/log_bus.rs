use crate::domain::log_entry::LogEntry;
use async_trait::async_trait;
use tokio::sync::broadcast;

/// Port for publishing log entries.
#[async_trait]
pub trait LogPublisher: Send + Sync {
    async fn publish(&self, log: LogEntry);
}

/// Port for subscribing to the log stream.
#[async_trait]
pub trait LogSubscriber: Send + Sync {
    /// Returns a receiver that will get a copy of all logs.
    fn subscribe(&self) -> broadcast::Receiver<LogEntry>;
}
