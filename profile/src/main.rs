mod pr_context;
mod pr_user;
mod pr_device;
mod pr_server;
mod pr_database;
mod pr_id_generator;
mod pr_device_handler;
mod pr_errors;
mod pr_spvr_client;
mod pr_settings;
mod pr_grpc_server;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing_subscriber::util::SubscriberInitExt;
use base::{log_util, RespMessage};
use crate::pr_context::PrContext;
use crate::pr_grpc_server::PrGrpcServer;
use crate::pr_server::PrServer;
use crate::pr_settings::PrSettings;
use crate::pr_spvr_client::PrSpvrClient;

lazy_static::lazy_static! {
    pub static ref gPrSpvrClient: Arc<Mutex<PrSpvrClient>> = Arc::new(Mutex::new(PrSpvrClient::new()));
    pub static ref gPrSettings: Arc<Mutex<PrSettings>> = Arc::new(Mutex::new(PrSettings::new()));
}

#[tokio::main]
async fn main() {
    let _guard = log_util::init_log("logs/profile/".to_string(), "log_profile".to_string());
    
    // settings
    gPrSettings.lock().await.load().await;
    
    // context
    let context = Arc::new(tokio::sync::Mutex::new(PrContext::new()));
    context.lock().await.init().await;

    // grpc server
    tokio::spawn(async move {
        PrGrpcServer::new().start().await;
    });
    
    // spvr client
    let spvr_srv_ip = gPrSettings.lock().await.spvr_server_ip.clone();
    let spvr_srv_port = gPrSettings.lock().await.spvr_server_port;
    let srv_id = gPrSettings.lock().await.server_id.clone();
    let address = format!("ws://{}:{}/inner?server_id={}&server_type=1", spvr_srv_ip, spvr_srv_port, srv_id);
    tracing::info!("connecting to: {}", address);
    gPrSpvrClient.lock().await.connect(address).await;
    
    // server
    let server = PrServer::new("0.0.0.0".to_string(), 
                               gPrSettings.lock().await.server_working_port as i32, context.clone());
    server.start().await;
}
