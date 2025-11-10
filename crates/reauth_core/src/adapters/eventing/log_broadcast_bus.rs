use crate::{
    domain::log_entry::LogEntry,
    ports::log_bus::{LogPublisher, LogSubscriber},
};
use async_trait::async_trait;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct LogBroadcastBus {
    sender: broadcast::Sender<LogEntry>,
}

impl LogBroadcastBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }
}

#[async_trait]
impl LogPublisher for LogBroadcastBus {
    async fn publish(&self, log: LogEntry) {
        self.sender.send(log).ok(); // Send, ignore error if no subscribers
    }
}

#[async_trait]
impl LogSubscriber for LogBroadcastBus {
    fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.sender.subscribe()
    }
}
