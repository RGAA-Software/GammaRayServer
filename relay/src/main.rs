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
mod relay_spvr_client;
mod relay_settings;
mod relay_statistics;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::relay_context::RelayContext;
use crate::relay_server::RelayServer;
use crate::relay_settings::RelaySettings;
use crate::relay_spvr_client::RelaySpvrClient;

lazy_static::lazy_static! {
    pub static ref gRelaySettings: Arc<Mutex<RelaySettings>> = Arc::new(Mutex::new(RelaySettings::new()));
}

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

    gRelaySettings.lock().await.init().await;

    let context = RelayContext::new().await;
    if let Err(err) = context {
        tracing::error!("Create RelayContext failed: {}", err);
        return;
    }
    let context = context.unwrap();
    let context = Arc::new(Mutex::new(context));
    context.lock().await.init();

    //
    tracing::info!("Starting RelayServer");
    let relay_grpc = Arc::new(Mutex::new(RelaySpvrClient::new()));
    relay_grpc.lock().await.connect().await;
    RelaySpvrClient::guard(relay_grpc.clone()).await;
    
    tracing::info!("after Starting RelayServer");

    let server = RelayServer::new("0.0.0.0".to_string(), 20481, context.clone());
    server.start().await;
}
