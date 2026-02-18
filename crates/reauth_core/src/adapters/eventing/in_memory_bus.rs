//! An in-process, Tokio-based event bus implementation.

use crate::{
    domain::events::DomainEvent,
    ports::event_bus::{EventHandler, EventPublisher, EventSubscriber},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

/// A Tokio `broadcast` channel-based event bus.
#[derive(Clone, Debug)]
pub struct InMemoryEventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024); // Channel with capacity
        Self { sender }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    /// Publishes an event to all subscribers.
    async fn publish(&self, event: DomainEvent) {
        if self.sender.send(event).is_err() {
            // This error means there are no subscribers.
            // In a simple in-process bus, this is normal and not a critical error.
        }
    }
}

#[async_trait]
impl EventSubscriber for InMemoryEventBus {
    /// Subscribes a new handler.
    /// This spawns a task to listen for events and forward them to the handler.
    async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        let mut receiver = self.sender.subscribe();

        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        // Forward the event to the specific handler
                        handler.handle(&event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        error!("EventBus: Subscriber is lagging behind.");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Sender was dropped, end the task
                        break;
                    }
                }
            }
        });
    }
}
