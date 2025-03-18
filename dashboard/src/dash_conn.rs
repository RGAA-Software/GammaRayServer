use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::stream::SplitSink;
use tokio::sync::Mutex;
use crate::dash_context::DashContext;

pub type DashConnPtr = Arc<Mutex<DashConn>>;

pub struct DashConn {
    pub context: Arc<Mutex<DashContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl DashConn {
    pub async fn new(context: Arc<Mutex<DashContext>>,
                     sender: Arc<Mutex<SplitSink<WebSocket, Message>>>) -> Self {
        DashConn {
            context,
            sender,
        }
    }

    pub async fn process_binary_message(&mut self, data: Bytes) -> bool {
        true
    }

    pub async fn process_text_message(&self, data: Utf8Bytes) -> bool {
        true
    }
}