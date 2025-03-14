use crate::gPrSettings;
use protocol::grpc_base::{HeartBeatReply, HeartBeatRequest};
use protocol::grpc_profile::grpc_profile_server::{GrpcProfile, GrpcProfileServer};
use protocol::grpc_relay::grpc_relay_server::{GrpcRelay, GrpcRelayServer};
use protocol::grpc_relay::{RelayQueryDeviceReply, RelayQueryDeviceRequest, RelayStreamReply, RelayStreamRequest};
use std::error::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Mutex};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status, Streaming};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<RelayStreamReply, Status>> + Send>>;

#[derive(Default)]
pub struct PrGrpcServer {

}

impl PrGrpcServer {
    pub fn new() -> Self {
        PrGrpcServer {
            
        }
    }

    pub async fn start(&self) {
        let server_grpc_port = gPrSettings.lock().await.server_grpc_port;
        let addr = format!("0.0.0.0:{}", server_grpc_port).parse().unwrap();
        let server = PrGrpcServer::default();
        tracing::info!("PrGrpcServer listening on {}", addr);
        let r = Server::builder()
            .add_service(GrpcProfileServer::new(server))
            .serve(addr)
            .await;
        if let Err(e) = r {
            tracing::error!("server error: {}", e);
        }
    }
}

#[tonic::async_trait]
impl GrpcProfile for PrGrpcServer {
    async fn heart_beat(&self, request: Request<HeartBeatRequest>) -> Result<Response<HeartBeatReply>, Status> {
        Ok(Response::new(HeartBeatReply {
            server_id: request.get_ref().server_id.clone(),
            hb_index: request.into_inner().hb_index,
        }))
    }
    
}

