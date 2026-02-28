use crate::domain::log::{LogEntry, LogPublisher, LogSubscriber};
use async_trait::async_trait;
use tokio::sync::broadcast;

pub struct LogBroadcastBus {
    sender: broadcast::Sender<LogEntry>,
}

impl LogBroadcastBus {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }
}

#[async_trait]
impl LogPublisher for LogBroadcastBus {
    async fn publish(&self, log: LogEntry) {
        let _ = self.sender.send(log);
    }
}

impl LogSubscriber for LogBroadcastBus {
    fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.sender.subscribe()
    }
}
