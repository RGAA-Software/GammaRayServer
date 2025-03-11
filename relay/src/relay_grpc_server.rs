use std::error::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Mutex};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use protocol::grpc_relay::grpc_relay_server::{GrpcRelay, GrpcRelayServer};
use protocol::grpc_base::{HeartBeatReply, HeartBeatRequest};
use protocol::grpc_relay::{RelayStreamRequest, RelayStreamResponse};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<RelayStreamResponse, Status>> + Send>>;

#[derive(Default)]
pub struct RelayGrpcServer {
}

impl RelayGrpcServer {
    pub fn new() -> Self {
        RelayGrpcServer {
        }
    }

    pub async fn start(&self) {
        let addr = "0.0.0.0:50051".parse().unwrap();
        let server = RelayGrpcServer::default();
        tracing::info!("GreeterServer listening on {}", addr);
        let r = Server::builder()
            .add_service(GrpcRelayServer::new(server))
            .serve(addr)
            .await;
        if let Err(e) = r {
            tracing::error!("server error: {}", e);
        }
    }
}

fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }

        err = err.source()?;
    }
}

#[tonic::async_trait]
impl GrpcRelay for RelayGrpcServer {
    async fn heart_beat(&self, request: Request<HeartBeatRequest>) -> Result<Response<HeartBeatReply>, Status> {
        Ok(Response::new(HeartBeatReply {
            server_id: request.get_ref().server_id.clone(),
            hb_index: request.into_inner().hb_index,
        }))
    }

    type StreamRequestStream = ResponseStream;

    async fn stream_request(
        &self,
        req: Request<Streaming<RelayStreamRequest>>,
    ) -> EchoResult<Self::StreamRequestStream> {
        println!("EchoServer::bidirectional_streaming_echo");

        let mut in_stream = req.into_inner();
        let (tx, rx) = mpsc::channel(128);

        // this spawn here is required if you want to handle connection error.
        // If we just map `in_stream` and write it back as `out_stream` the `out_stream`
        // will be dropped when connection error occurs and error will never be propagated
        // to mapped version of `in_stream`.
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => tx
                        .send(Ok(RelayStreamResponse {
                            server_id: "".to_string(),
                            message: v.message
                        }))
                        .await
                        .expect("working rx"),
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                eprintln!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }

                        match tx.send(Err(err)).await {
                            Ok(_) => (),
                            Err(_err) => break, // response was dropped
                        }
                    }
                }
            }
            println!("\tstream ended");
        });

        // echo just write the same data that was received
        let out_stream = ReceiverStream::new(rx);


        Ok(Response::new(
            Box::pin(out_stream) as Self::StreamRequestStream
        ))
    }
}