mod pr_context;
mod pr_user;
mod pr_device;
mod pr_server;
mod pr_database;
mod pr_id_generator;
mod pr_device_handler;
mod pr_errors;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::util::SubscriberInitExt;
use base::{log_util, RespMessage};
use crate::pr_context::PrContext;
use crate::pr_server::PrServer;

#[tokio::main]
async fn main() {
    let _guard = log_util::init_log("logs/profile/".to_string(), "log_profile".to_string());

    let context = Arc::new(tokio::sync::Mutex::new(PrContext::new()));
    context.lock().await.init().await;
    
    let server = PrServer::new("0.0.0.0".to_string(), 20581, context.clone());
    server.start().await;
}
