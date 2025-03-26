use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use axum::body::Bytes;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Value};
use tokio::sync::Mutex;
use crate::relay_conn::RelayConn;

pub struct RelayConnManager {
    pub relay_conns: Mutex<HashMap<String, Arc<Mutex<RelayConn>>>>,
}

impl RelayConnManager {
    pub fn new() -> RelayConnManager {
        RelayConnManager {
            relay_conns: Mutex::new(HashMap::new()),
        }
    }
    
    pub async fn add_connection(&mut self, device_id: String, relay_conn: Arc<Mutex<RelayConn>>) {
        self.relay_conns.lock().await.insert(device_id.clone(), relay_conn);
    }
    
    pub async fn remove_connection(&self, device_id: String) {
        self.relay_conns.lock().await.remove(&device_id);
    }

    pub async fn get_connection(&self, device_id: String) -> Option<Arc<Mutex<RelayConn>>> {
        self.relay_conns.lock().await.get(&device_id).cloned()
    }
}