mod spvr_context;
mod spvr_server;
mod spvr_relay_client;
mod spvr_settings;
mod spvr_conn;

use std::collections::HashMap;
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
    pub static ref gSpvrRelayClients: Mutex<HashMap<String, Arc<Mutex<SpvrRelayClient>>>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() {
    // log
    let _guard = log_util::init_log("logs/supervisor/".to_string(), "log_supervisor".to_string());

    // settings
    gSpvrSettings.lock().await.load();

    // context
    let context = Arc::new(Mutex::new(spvr_context::SpvrContext::new()));

    // relay grpc client
    //let relay_servers_config = gSpvrSettings.lock().await.relay_servers.clone();
    //for config in relay_servers_config {
        // let grpc_relay_client = Arc::new(Mutex::new(SpvrRelayClient::new()));
        // grpc_relay_client.lock().await.connect(config.clone()).await;
        // SpvrRelayClient::guard(grpc_relay_client.clone()).await;
        // gSpvrRelayClients.lock().await.insert(config.ip.clone(), grpc_relay_client);
        // tracing::info!("after Starting RelayServer");
    //}
    
    // server
    let server = SpvrServer::new("0.0.0.0".to_string(),
                                 gSpvrSettings.lock().await.server_port,
                                 context);
    server.start().await;
}
