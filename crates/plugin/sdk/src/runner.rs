//! Contains the main `run` function to bootstrap a plugin.

use crate::{
    constants,
    grpc::plugin::v1::{
        handshake_server::{Handshake, HandshakeServer},
        Empty, PluginInfo,
    },
    plugin::Plugin,
};
use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

/// The helper function that bootstraps a plugin.
///
/// This function handles all the boilerplate of starting a gRPC server,
/// adding the required `Handshake` service, printing the handshake string to stdout,
/// and keeping the plugin process alive.
///
/// # Arguments
/// * `plugin` - A struct that implements the `Plugin` trait for metadata.
/// * `add_services` - A closure that receives the tonic `Router` and adds all custom gRPC services.
pub async fn run<P, F>(plugin: P, add_services: F) -> anyhow::Result<()>
where
    P: Plugin + Send + Sync + 'static,
    F: FnOnce(tonic::transport::server::Router) -> tonic::transport::server::Router + Send + 'static,
{
    let listener = tokio::net::TcpListener::bind(constants::PLUGIN_SERVER_BIND_ADDR).await?;
    let addr = listener.local_addr()?;

    let handshake_service = HandshakeService {
        plugin: Arc::new(plugin),
    };

    // Start the server builder, adding the mandatory handshake
    let router = Server::builder()
        .add_service(HandshakeServer::new(handshake_service));

    // Call the user's closure to add their custom services
    let router_with_services = add_services(router);

    let server_task = tokio::spawn(async move {
        router_with_services // Use the final, configured router
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
    });

    let handshake_line = format!(
        "{}|{}|{}|{}|{}",
        constants::HANDSHAKE_CORE_VERSION,
        constants::HANDSHAKE_PROTOCOL_VERSION,
        constants::HANDSHAKE_PROTOCOL_NETWORK,
        addr,
        constants::HANDSHAKE_PROTOCOL_TYPE
    );
    println!("{}", handshake_line);
    tracing::info!("[Plugin SDK] Plugin listening on {}", addr);

    server_task.await??;
    Ok(())
}

/// An internal struct to implement the required `Handshake` gRPC service.
struct HandshakeService<P: Plugin> {
    plugin: Arc<P>,
}

#[tonic::async_trait]
impl<P: Plugin + Send + Sync + 'static> Handshake for HandshakeService<P> {
    async fn get_plugin_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PluginInfo>, Status> {
        Ok(Response::new(self.plugin.info()))
    }
}