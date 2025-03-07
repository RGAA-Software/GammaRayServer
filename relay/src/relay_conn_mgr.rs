use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use axum::body::Bytes;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Value};
use tokio::sync::Mutex;
use crate::proto::tc::RelayMessage;
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
    
    pub async fn add_connection(&mut self, device_id: &String, relay_conn: Arc<Mutex<RelayConn>>) {
        self.relay_conns.lock().await.insert(device_id.clone(), relay_conn);
        // let conn_redis_id = format!("relay_conn:{}", device_id.clone());
        // let conn_info = [
        //     ("device_id", conn_redis_id.clone()),
        //     ("connected_timestamp", base::get_current_timestamp().to_string()),
        //     ("last_update_timestamp", base::get_current_timestamp().to_string()),
        // ];
        //
        // let r = self.redis_conn
        //     .lock().await
        //     .hset_multiple::<String, &str, String, ()>(conn_redis_id, &conn_info).await;
    }
    
    pub async fn remove_connection(&self, device_id: &String) {
        self.relay_conns.lock().await.remove(device_id);
        // let conn_redis_id = format!("relay_conn:{}", device_id.clone());
        // let r = self.redis_conn
        //     .lock().await
        //     .del::<String, ()>(conn_redis_id).await;
    }

    pub async fn get_connection(&self, device_id: &String) -> Option<Arc<Mutex<RelayConn>>> {
        self.relay_conns.lock().await.get(device_id).cloned()
    }
}