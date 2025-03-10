use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{Json, Router};
use axum::routing::{any, get};
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use base::RespMessage;
use crate::spvr_context::SpvrContext;

pub struct SpvrServer {
    pub host: String,
    pub port: u16,
    pub context: Arc<Mutex<SpvrContext>>,
}

impl SpvrServer {
    pub fn new(host: String, port: u16, context: Arc<Mutex<SpvrContext>>) -> Self {
        SpvrServer {
            host,
            port,
            context,
        }
    }

    pub async fn start(&self) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

        let app = Router::new()
            .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
            .route("/", get(SpvrServer::root))
            .with_state(self.context.clone());
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        axum::serve(listener,  app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    }
    
    pub async fn root() -> Json<RespMessage<String>> {
        Json(base::ok_resp_str("Working".to_string()))
    }
}