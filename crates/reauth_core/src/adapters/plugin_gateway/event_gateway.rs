use crate::{domain::events::DomainEvent, ports::event_bus::EventHandler};
use manager::PluginManager;

use manager::grpc::plugin::v1::event_listener_client::EventListenerClient;
use manager::grpc::plugin::v1::EventRequest;

pub struct PluginEventGateway {
    plugin_manager: PluginManager,
}

impl PluginEventGateway {
    pub fn new(plugin_manager: PluginManager) -> Self {
        Self { plugin_manager }
    }
}

#[async_trait::async_trait]
impl EventHandler for PluginEventGateway {
    async fn handle(&self, event: &DomainEvent) {
        let event_type = event.event_type().to_string();
        let payload = event.payload_json();

        // --- THIS IS THE REFACTOR ---
        // Get a list of all *running* plugins from the manager.
        let active_plugins = self.plugin_manager.get_all_active_plugins().await;

        for (manifest, channel) in active_plugins {
            // Check if this plugin's manifest is subscribed to this event type
            if manifest.events.subscribes_to.contains(&event_type) {
                // Make the direct gRPC call
                let mut client = EventListenerClient::new(channel); // Use the provided channel
                let request = tonic::Request::new(EventRequest {
                    event_type: event_type.clone(),
                    event_payload_json: payload.clone(),
                });

                // Send the event but don't wait for a response
                tokio::spawn(async move {
                    let _ = client.on_event(request).await;
                });
            }
        }
        // --- END REFACTOR ---
    }
}
