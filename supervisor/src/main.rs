mod spvr_context;
mod spvr_server;
mod spvr_grpc_relay_client;
mod spvr_settings;
mod spvr_conn;
mod spvr_conn_mgr;
mod spvr_grpc_profile_client;
mod spvr_grpc_relay_client_mgr;
mod spvr_grpc_profile_client_mgr;
mod spvr_grpc_client_mgr_trait;
mod spvr_handler_server;
mod spvr_defs;
mod spvr_handler_device;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use base::log_util;
use crate::spvr_conn_mgr::SpvrConnManager;
use crate::spvr_grpc_profile_client_mgr::SpvrGrpcProfileClientMgr;
use crate::spvr_grpc_relay_client_mgr::SpvrGrpcRelayClientManager;
use crate::spvr_grpc_relay_client::SpvrGrpcRelayClient;
use crate::spvr_server::SpvrServer;
use crate::spvr_settings::SpvrSettings;

lazy_static::lazy_static! {
    pub static ref gSpvrSettings: Arc<Mutex<SpvrSettings>> = Arc::new(Mutex::new(SpvrSettings::new()));
    pub static ref gSpvrConnMgr: Arc<Mutex<SpvrConnManager>> = Arc::new(Mutex::new(SpvrConnManager::new()));
    pub static ref gSpvrGrpcRelayClientMgr: Arc<Mutex<SpvrGrpcRelayClientManager >> = Arc::new(Mutex::new(SpvrGrpcRelayClientManager::new()));
    pub static ref gSpvrGrpcProfileClientMgr: Arc<Mutex<SpvrGrpcProfileClientMgr >> = Arc::new(Mutex::new(SpvrGrpcProfileClientMgr::new()));
}

#[tokio::main]
async fn main() {
    // log
    let _guard = log_util::init_log("logs/supervisor/".to_string(), "log_supervisor".to_string());

    // settings
    gSpvrSettings.lock().await.load();

    // context
    let context = Arc::new(Mutex::new(spvr_context::SpvrContext::new()));
    
    // server
    let server = SpvrServer::new("0.0.0.0".to_string(),
                                 gSpvrSettings.lock().await.server_port,
                                 context);
    server.start().await;
}
