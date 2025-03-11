use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
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
}