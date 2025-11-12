use sdk::{
    plugin::Plugin,
    prelude::v1::{
        event_listener_server::{EventListener, EventListenerServer},
        greeter_server::{Greeter, GreeterServer},
        EventRequest, EventResponse, HelloReply, HelloRequest, PluginInfo,
    },
    runner::run,
};
use tonic::{Request, Response, Status};
use tracing::info;

pub struct HelloWorldPlugin;
impl Plugin for HelloWorldPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Hello World Plugin".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct GreeterService;

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        info!(request.name = %name, "Received 'say_hello' request");
        Ok(Response::new(HelloReply {
            message: format!(
                "Hello, {}! This message is from the Rust plugin backend.",
                name
            ),
        }))
    }
}

#[derive(Default, Clone)]
pub struct MyEventListener;

#[tonic::async_trait]
impl EventListener for MyEventListener {
    async fn on_event(
        &self,
        request: Request<EventRequest>,
    ) -> Result<Response<EventResponse>, Status> {
        let event = request.into_inner();

        info!(event.type = %event.event_type, event.payload = %event.event_payload_json,
            "Received event from Core"
        );

        Ok(Response::new(EventResponse {}))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .json()
        .init();

    info!("Plugin backend starting up...");

    let plugin = HelloWorldPlugin;
    let greeter_service = GreeterService::default();
    let event_listener_service = MyEventListener::default();

    run(plugin, |router| {
        router
            .add_service(GreeterServer::new(greeter_service))
            .add_service(EventListenerServer::new(event_listener_service))
    })
    .await?;

    Ok(())
}
