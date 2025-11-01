//! Defines the interfaces (ports) for the application's event bus.

use crate::domain::events::DomainEvent;
use async_trait::async_trait;
use std::sync::Arc;

/// The port for any service that needs to publish events.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent);
}

/// The port for the event bus system to register listeners.
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Subscribes a handler to listen for all events.
    async fn subscribe(&self, handler: Arc<dyn EventHandler>);
}

/// The trait all event listeners must implement.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handles an incoming event.
    async fn handle(&self, event: &DomainEvent);
}