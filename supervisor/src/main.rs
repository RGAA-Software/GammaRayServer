mod spvr_context;
mod spvr_server;
mod spvr_relay_client;
mod spvr_settings;

use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use base::log_util;
use crate::spvr_relay_client::SpvrRelayClient;
use crate::spvr_server::SpvrServer;
use crate::spvr_settings::SpvrSettings;

lazy_static::lazy_static! {
    pub static ref gSpvrSettings: Arc<Mutex<SpvrSettings>> = Arc::new(Mutex::new(SpvrSettings::new()));
    pub static ref gSpvrRelayClient: Arc<Mutex<SpvrRelayClient>> = Arc::new(Mutex::new(SpvrRelayClient::new()));
}

#[tokio::main]
async fn main() {
    // log
    let _guard = log_util::init_log("logs/supervisor/".to_string(), "log_supervisor".to_string());

    // context
    let context = Arc::new(Mutex::new(spvr_context::SpvrContext::new()));

    // relay grpc client
    gSpvrRelayClient.lock().await.connect().await;
    SpvrRelayClient::guard(gSpvrRelayClient.clone()).await;
    tracing::info!("after Starting RelayServer");

    // server
    let server = SpvrServer::new("0.0.0.0".to_string(), 20582, context);
    server.start().await;
}
