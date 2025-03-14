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
use crate::pr_server::PrServer;
use crate::pr_spvr_client::PrSpvrClient;

lazy_static::lazy_static! {
    pub static ref gPrSpvrClient: Arc<Mutex<PrSpvrClient>> = Arc::new(Mutex::new(PrSpvrClient::new()));
}

#[tokio::main]
async fn main() {
    let _guard = log_util::init_log("logs/profile/".to_string(), "log_profile".to_string());

    let context = Arc::new(tokio::sync::Mutex::new(PrContext::new()));
    context.lock().await.init().await;

    gPrSpvrClient.lock().await.connect("ws://127.0.0.1:30500/inner?server_id=pr_01&server_type=1".to_string()).await;
    
    let server = PrServer::new("0.0.0.0".to_string(), 20581, context.clone());
    server.start().await;
}
