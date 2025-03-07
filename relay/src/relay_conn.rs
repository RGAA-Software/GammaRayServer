use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use redis::AsyncCommands;
use tokio::sync::Mutex;
use crate::proto::tc::RelayMessage;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_context::RelayContext;

pub struct RelayConn {
    pub context: Arc<Mutex<RelayContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub conn_mgr: Arc<Mutex<RelayConnManager>>,
    pub device_id: String,
    pub remote_device_id: Option<String>,
    pub last_update_timestamp: i64,
}

impl RelayConn {
    pub async fn new(context: Arc<Mutex<RelayContext>>,
               sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
               device_id: String) -> Arc<Mutex<RelayConn>> {
        let conn_mgr = context.lock().await.conn_mgr.clone();
        Arc::new(Mutex::new(RelayConn {
            context,
            sender,
            conn_mgr,
            device_id,
            remote_device_id: None,
            last_update_timestamp: base::get_current_timestamp(),
        }))
    }

    pub async fn append_received_data_size(&mut self, size: usize) {
        // to redis; key: year/month/
        let key = format!("{}", "".to_string());
        let r = self.context.lock().await
            .redis_conn.lock().await
            .set::<String, String, ()>("".to_string(), "".to_string()).await;
        if let Err(e) = r {
            tracing::error!("update received data for: {} failed", key);
        }
    }

    pub async fn append_send_data_size(&mut self, size: usize) {
        // to redis
        let key = format!("{}", "".to_string());
        let r = self.context.lock().await
            .redis_conn.lock().await
            .set::<String, String, ()>("".to_string(), "".to_string()).await;
        if let Err(e) = r {
            tracing::error!("update send data for: {} failed", key);
        }
    }

    pub fn on_hello(&mut self, value: serde_json::Value) {
        self.last_update_timestamp = base::get_current_timestamp();
    }

    pub fn on_heartbeat(&mut self, value: serde_json::Value) {
        self.last_update_timestamp = base::get_current_timestamp();
    }

    pub async fn on_remote_error(&self, value: serde_json::Value) {

    }

    pub async fn on_create_room(&self, value: serde_json::Value) {

    }

    pub async fn on_request_control(&self, value: serde_json::Value) {

    }

    pub async fn on_request_control_resp(&self, value: serde_json::Value) {

    }

    pub async fn on_relay(&mut self, om: Bytes) {
        // append received data size
        self.append_received_data_size(om.len()).await;

        // 1. find paired device
        if let None = self.remote_device_id {
            return;
        }

        let remote_device_id = self.remote_device_id.clone().unwrap();
        let remote_device = self.conn_mgr.lock().await
            .get_connection(&remote_device_id).await;
        if remote_device.is_none() {
            return;
        }

        // 2. relay to this device
        let paired_device = remote_device.unwrap();
        let r = paired_device.lock().await.send_message(om).await;
        if !r {
            tracing::error!("on_relay failed: {} -> {}", self.device_id, remote_device_id);
        }
    }

    pub fn set_remote_device_id(&mut self, device_id: &String) {
        self.remote_device_id = Some(device_id.clone());
    }

    pub fn clear_remote_device_id(&mut self) {
        self.remote_device_id = None;
    }

    // send back to this connection itself.
    pub async fn send_message(&mut self, om: Bytes) -> bool {
        let size = om.len();
        let r = self.sender.lock().await.send(Message::Binary(om)).await;
        if let Err(r) = r {
            tracing::error!("error sending relay message: {r}");
            return false;
        }
        self.append_send_data_size(size).await;
        true
    }
}