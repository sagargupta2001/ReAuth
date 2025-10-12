use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::Body;
use tonic::server::NamedService;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tower::Service;

// Re-export generated types for plugin authors to use
pub mod proto {
    pub mod plugin {
        pub mod v1 {
            tonic::include_proto!("plugin.v1");
        }
    }
}
use proto::plugin::v1::{
    handshake_server::{Handshake, HandshakeServer},
    Empty, PluginInfo,
};

/// The main trait a plugin's metadata struct must implement.
pub trait Plugin {
    fn info(&self) -> PluginInfo;
}

/// The helper function that bootstraps a plugin.
pub async fn run<P, S>(plugin: P, service: S) -> anyhow::Result<()>
where
    P: Plugin + Send + Sync + 'static,
    S: Service<
        http::Request<Body>,
        Response = http::Response<BoxBody>,
        Error = std::convert::Infallible,
    > + NamedService
    + Clone
    + Send
    + 'static,
    S::Future: Send + 'static,
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let handshake_service = HandshakeService { plugin: Arc::new(plugin) };

    let server_task = tokio::spawn(async move {
        Server::builder()
            .add_service(HandshakeServer::new(handshake_service))
            .add_service(service)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
    });

    let handshake_line = format!("1|1|tcp|{}|grpc", addr);
    println!("{}", handshake_line);
    tracing::info!("[Plugin SDK] Plugin listening on {}", addr);

    server_task.await??;
    Ok(())
}

// Internal struct to implement the required Handshake gRPC service.
struct HandshakeService<P: Plugin> {
    plugin: Arc<P>,
}

#[tonic::async_trait]
impl<P: Plugin + Send + Sync + 'static> Handshake for HandshakeService<P> {
    async fn get_plugin_info(&self, _request: Request<Empty>) -> Result<Response<PluginInfo>, Status> {
        Ok(Response::new(self.plugin.info()))
    }
}