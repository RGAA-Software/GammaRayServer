use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
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
            .route("/ping", get(PrServer::ping))
            .route("/create/new/device", post(PrDeviceHandler::create_new_device))
            .route("/verify/device/info", get(PrDeviceHandler::verify_device_info))
            .route("/query/devices", get(PrDeviceHandler::query_devices))
            .route("/append/used/time", post(PrDeviceHandler::append_used_time))
            .route("/update/random/pwd", post(PrDeviceHandler::update_random_pwd))
            .with_state(self.context.clone());
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    pub async fn ping(State(ctx): State<Arc<Mutex<PrContext>>>) -> Json<RespMessage<String>> {
        Json(RespMessage::<String> {
            code: 200,
            message: "ok".to_string(),
            data: "Pong".to_string(),
        })
    }
}