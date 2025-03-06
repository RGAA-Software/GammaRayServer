use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use base::RespMessage;
use crate::pr_context::PrContext;
use crate::pr_device_handler::PrDeviceHandler;

pub struct PrServer {
    pub host: String,
    pub port: i32,
    pub context: Arc<tokio::sync::Mutex<PrContext>>,
}

impl PrServer {

    pub fn new(host: String, port: i32, context: Arc<tokio::sync::Mutex<PrContext>>) -> PrServer {
        PrServer {
            host,
            port,
            context
        }
    }

    pub async fn start(&self) {
        // build our application with a route
        let app = Router::new()
            // `GET /` goes to `root`
            .route("/", get(PrServer::root))
            .route("/create/new/device", post(PrDeviceHandler::create_new_device))
            .route("/query/devices", get(PrDeviceHandler::query_devices))
            .route("/append/used/time", post(PrDeviceHandler::append_used_time))
            .with_state(self.context.clone());
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    pub async fn root(State(ctx): State<Arc<tokio::sync::Mutex<PrContext>>>) -> &'static str {
        "Hello, World!"
    }

}