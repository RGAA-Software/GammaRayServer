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
use base::RespMessage;
use crate::pr_context::PrContext;
use crate::pr_server::PrServer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let context = Arc::new(tokio::sync::Mutex::new(PrContext::new()));
    context.lock().await.init().await;
    
    let server = PrServer::new("0.0.0.0".to_string(), 3000, context.clone());
    server.start().await;
}
