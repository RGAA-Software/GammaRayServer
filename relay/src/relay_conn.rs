use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use tokio::sync::Mutex;
use crate::proto::tc::RelayMessage;
use crate::relay_context::RelayContext;

pub struct RelayConn {
    pub context: Arc<Mutex<RelayContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub device_id: String,
}

impl RelayConn {
    pub fn new(context: Arc<Mutex<RelayContext>>,
               sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
               device_id: String) -> Arc<Mutex<RelayConn>> {
        Arc::new(Mutex::new(RelayConn {
            context,
            sender,
            device_id,
        }))
    }

    pub async fn on_hello(&self) {

    }

    pub async fn on_heartbeat(&self) {

    }

    pub async fn on_error(&self, rm: RelayMessage) {

    }

    pub async fn on_relay(&self, rm: RelayMessage, om: Bytes) {
        let bytes = "".into();
        let r = self.sender.lock().await.send(Message::Binary(bytes)).await;
    }

    pub async fn on_create_room(&self, rm: RelayMessage, om: Bytes) {

    }

    pub async fn on_request_control(&self, rm: RelayMessage, om: Bytes) {

    }

    pub async fn on_request_control_resp(&self, rm: RelayMessage, om: Bytes) {

    }
}