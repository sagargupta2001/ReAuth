use crate::domain::events::DomainEvent;
use crate::ports::event_bus::EventPublisher;
use async_trait::async_trait;
use std::sync::Arc;

/// Fan-out publisher that forwards events to multiple publishers.
#[derive(Clone)]
pub struct MultiEventPublisher {
    publishers: Vec<Arc<dyn EventPublisher>>,
}

impl MultiEventPublisher {
    pub fn new(publishers: Vec<Arc<dyn EventPublisher>>) -> Self {
        Self { publishers }
    }
}

#[async_trait]
impl EventPublisher for MultiEventPublisher {
    async fn publish(&self, event: DomainEvent) {
        for publisher in &self.publishers {
            publisher.publish(event.clone()).await;
        }
    }
}
