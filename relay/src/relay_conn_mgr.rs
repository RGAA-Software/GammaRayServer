use std::collections::HashMap;
use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Value};
use tokio::sync::Mutex;

pub struct RelayConnManager {
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RelayConnManager {
    pub fn new(redis_conn: Arc<Mutex<MultiplexedConnection>>,) -> RelayConnManager {
        RelayConnManager {
            redis_conn
        }
    }
    
    pub async fn add_connection(&self, device_id: &String) {
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
}