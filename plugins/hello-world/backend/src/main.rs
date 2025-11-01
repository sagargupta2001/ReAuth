
use tonic::{Request, Response, Status};
use sdk::plugin::Plugin;
use sdk::prelude::v1::greeter_server::{Greeter, GreeterServer};
use sdk::prelude::v1::{HelloReply, HelloRequest, PluginInfo};
use sdk::runner::run;

// 1. Define the plugin's main struct
pub struct HelloWorldPlugin;
impl Plugin for HelloWorldPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Hello World Plugin".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

// 2. Implement the gRPC service
#[derive(Default, Clone)]
pub struct GreeterService;

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        println!("[Plugin Backend] Received a 'say_hello' request for '{}'", name);
        Ok(Response::new(HelloReply {
            message: format!("Hello, {}! This message is from the Rust plugin backend (1/11/2025)!.", name),
        }))
    }
}

// 3. Main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let plugin = HelloWorldPlugin;
    let greeter_service = GreeterService::default();

    run(plugin, GreeterServer::new(greeter_service)).await?;

    Ok(())
}