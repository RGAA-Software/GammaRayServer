mod relay_context;
mod relay_server;
mod relay_conn_mgr;
mod relay_conn;
mod relay_room_mgr;
mod relay_message;
mod relay_room;
mod relay_proto_maker;
mod relay_room_handler;
mod relay_errors;
mod relay_settings;
mod relay_statistics;
mod relay_grpc_server;
mod relay_spvr_client;

use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, filter::LevelFilter};
use tracing_subscriber::prelude::*;
use base::log_util;
use crate::relay_context::RelayContext;
use crate::relay_grpc_server::RelayGrpcServer;
use crate::relay_server::RelayServer;
use crate::relay_settings::RelaySettings;
use crate::relay_spvr_client::RelaySpvrClient;

lazy_static::lazy_static! {
    pub static ref gRelaySettings: Arc<Mutex<RelaySettings>> = Arc::new(Mutex::new(RelaySettings::new()));
    pub static ref gRelayGrpcServer: Arc<Mutex<RelayGrpcServer>> = Arc::new(Mutex::new(RelayGrpcServer::new()));
    pub static ref gRelaySpvrClient: Arc<Mutex<RelaySpvrClient>> = Arc::new(Mutex::new(RelaySpvrClient::new()));
}

#[tokio::main]
async fn main() {
    let _guard = log_util::init_log("logs/relay/".to_string(), "log_relay".to_string());

    gRelaySettings.lock().await.load().await;

    let context = RelayContext::new().await;
    if let Err(err) = context {
        tracing::error!("Create RelayContext failed: {}", err);
        return;
    }
    let context = context.unwrap();
    let context = Arc::new(Mutex::new(context));
    context.lock().await.init();

    // grpc relay server
    // std::thread::spawn(|| {
    //     let rt = Runtime::new().expect("Failed to create Tokio runtime");
    //     rt.block_on(async move {
    //         tracing::info!("will start grpc relay server.");
    //         let grpc_relay = RelayGrpcServer::new();
    //         grpc_relay.start().await;
    //         tracing::info!("after grpc relay server.");
    //     });
    // });

    tokio::spawn(async move {
        tracing::info!("will start grpc relay server.");
        let grpc_relay = RelayGrpcServer::new();
        grpc_relay.start().await;
        tracing::info!("after grpc relay server.");
    });

    gRelaySpvrClient.lock().await.connect(format!("ws://{}:{}/inner?server_id={}&server_type=0",
        gRelaySettings.lock().await.spvr_server_ip, gRelaySettings.lock().await.spvr_server_port,
        gRelaySettings.lock().await.server_id)
    ).await;

    let server = RelayServer::new("0.0.0.0".to_string(),
                                  gRelaySettings.lock().await.server_working_port,
                                  context.clone());
    server.start().await;
}
