use std::ops::ControlFlow;
use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::stream::SplitSink;
use tokio::sync::Mutex;
use crate::spvr_context::SpvrContext;

pub struct SpvrConn {
    pub context: Arc<Mutex<SpvrContext>>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl SpvrConn {
    pub async fn new(context: Arc<Mutex<SpvrContext>>,
               sender: Arc<Mutex<SplitSink<WebSocket, Message>>>) -> SpvrConn {
        SpvrConn {
            context,
            sender,
        }
    }
    
    pub async fn process_binary_message(&self, m: Bytes) -> bool {
        false
    }
    
    pub async fn process_text_message(&self, data: Utf8Bytes) -> bool {
        let value: serde_json::error::Result<serde_json::Value> = serde_json::from_str(data.as_str());
        if let Err(e) = value {
            tracing::error!("parse json error: {e}, json: {}", data.to_string());
            return false;
        }
        
        true
    }
}