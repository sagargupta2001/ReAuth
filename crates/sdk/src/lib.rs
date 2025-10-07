use std::{sync::Arc};
use tonic::{Request, Response, Status};

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

pub trait Plugin {
    fn info(&self) -> PluginInfo;
}


pub async fn run<P, F>(plugin: P, add_services: F) -> anyhow::Result<()>
where
    P: Plugin + Send + Sync + 'static,
    F: FnOnce(tonic::transport::server::Router) -> tonic::transport::server::Router + Send + 'static,
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let handshake_service = HandshakeService {
        plugin: Arc::new(plugin),
    };

    let router = tonic::transport::Server::builder()
        .add_service(HandshakeServer::new(handshake_service));

    let router = add_services(router);

    let server_task = tokio::spawn(async move {
        router
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
    });

    let handshake_line = format!("1|1|tcp|{}|grpc", addr);
    println!("{}", handshake_line);
    tracing::info!("Plugin listening on {}", addr);

    server_task.await??;
    Ok(())
}

struct HandshakeService<P: Plugin> {
    plugin: Arc<P>,
}

#[tonic::async_trait]
impl<P> Handshake for HandshakeService<P>
where
    P: Plugin + Send + Sync + 'static,
{
    async fn get_plugin_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PluginInfo>, Status> {
        Ok(Response::new(self.plugin.info()))
    }
}
