use tonic::{Request, Response, Status};
use sdk::{run, Plugin};
use sdk::proto::plugin::v1::greeter_server::{Greeter, GreeterServer};
use sdk::proto::plugin::v1::{HelloReply, HelloRequest, PluginInfo};

// 1. Define the plugin
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
#[derive(Default)]
pub struct GreeterService;

#[tonic::async_trait]
impl Greeter for GreeterService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = request.into_inner().name;
        Ok(Response::new(HelloReply {
            message: format!("Hello, {}! This message is from the Rust plugin backend.", name),
        }))
    }
}

// 3. Main function
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let plugin = HelloWorldPlugin;
    let greeter_service = GreeterService::default();

    // Pass a closure that receives a tonic Router
    run(plugin, |mut router: tonic::transport::server::Router| {
        router = router.add_service(GreeterServer::new(greeter_service));
        router
    })
        .await?;

    Ok(())
}
