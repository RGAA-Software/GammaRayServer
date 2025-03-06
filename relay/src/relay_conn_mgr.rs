use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use axum::body::Bytes;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Value};
use tokio::sync::Mutex;

pub struct RelayConnManager {
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
    pub sender: Option<Arc<Mutex<SplitSink<WebSocket, Message>>>>,
}

impl RelayConnManager {
    pub fn new(redis_conn: Arc<Mutex<MultiplexedConnection>>,) -> RelayConnManager {
        RelayConnManager {
            redis_conn,
            sender: None,
        }
    }
    
    pub async fn add_connection(&mut self, device_id: &String, sender: Arc<Mutex<SplitSink<WebSocket, Message>>>) {
        self.sender = Some(sender);
        let conn_redis_id = format!("relay_conn:{}", device_id.clone());
        let conn_info = [
            ("device_id", conn_redis_id.clone()),
            ("connected_timestamp", base::get_current_timestamp().to_string()),
            ("last_update_timestamp", base::get_current_timestamp().to_string()),
        ];

        let r = self.redis_conn
            .lock().await
            .hset_multiple::<String, &str, String, ()>(conn_redis_id, &conn_info).await;
    }
    
    pub async fn remove_connection(&self, device_id: &String) {
        let conn_redis_id = format!("relay_conn:{}", device_id.clone());
        let r = self.redis_conn
            .lock().await
            .del::<String, ()>(conn_redis_id).await;
    }

    pub async fn on_hello(&self) {

    }

    pub async fn on_heartbeat(&self) {

    }

    pub async fn on_relay(&self, m: Bytes) {
        if let Some(sender) = &self.sender {
            let bytes = "".into();
            let r = sender.lock().await.send(Message::Binary(bytes)).await;
        }
    }
}