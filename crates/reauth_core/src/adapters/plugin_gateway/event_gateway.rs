use crate::{
    domain::events::DomainEvent,
    ports::event_bus::EventHandler,
};
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
        let (event_type, payload) = event.to_serializable();

        // Get a list of all running plugins
        let mut instances = self.plugin_manager.instances.lock().await;

        for instance in instances.values_mut() {
            // Check if this plugin's manifest is subscribed to this event type
            if instance.manifest.events.subscribes_to.contains(&event_type) {

                // Make the direct gRPC call
                let mut client = EventListenerClient::new(instance.grpc_channel.clone());
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
    }
}

impl DomainEvent {
    /// Helper function to serialize events for gRPC.
    /// Returns a tuple of (event_type, event_payload_json).
    pub fn to_serializable(&self) -> (String, String) {
        match self {
            DomainEvent::UserCreated(e) => (
                "UserCreated".to_string(),
                serde_json::to_string(e).unwrap_or_else(|_| "{}".to_string()),
            ),
            DomainEvent::UserAssignedToGroup(e) => (
                "UserAssignedToGroup".to_string(),
                serde_json::to_string(e).unwrap_or_else(|_| "{}".to_string()),
            ),
            DomainEvent::RoleAssignedToGroup(e) => (
                "RoleAssignedToGroup".to_string(),
                serde_json::to_string(e).unwrap_or_else(|_| "{}".to_string()),
            ),
            DomainEvent::RolePermissionChanged(e) => (
                "RolePermissionChanged".to_string(),
                serde_json::to_string(e).unwrap_or_else(|_| "{}".to_string()),
            ),
        }
    }
}
