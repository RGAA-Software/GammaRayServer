mod spvr_context;
mod spvr_server;
mod spvr_grpc_relay;

use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::spvr_server::SpvrServer;
use crate::spvr_grpc_relay::SpvrGrpcRelayServer;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let context = Arc::new(Mutex::new(spvr_context::SpvrContext::new()));

    // grpc relay server
    std::thread::spawn(|| {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async move {
            tracing::info!("will start grpc relay server.");
            let grpc_relay = SpvrGrpcRelayServer::new();
            grpc_relay.start().await;
            tracing::info!("after grpc relay server.");
        });
    });

    let server = SpvrServer::new("0.0.0.0".to_string(), 20582, context);
    server.start().await;
}
