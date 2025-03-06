use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use crate::relay_conn_mgr::RelayConnManager;

pub struct RelayContext {
    pub device_conn_mgr: Arc<Mutex<RelayConnManager>>,
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RelayContext {
    pub async fn new() -> Result<RelayContext, String> {
        let redis_client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
        let redis_conn = redis_client.get_multiplexed_async_connection().await;
        if let Err(err) = redis_conn {
            return Err(err.to_string());
        }
        let redis_conn = redis_conn.unwrap();
        let redis_conn = Arc::new(Mutex::new(redis_conn));
        Ok(RelayContext {
            device_conn_mgr: Arc::new(Mutex::new(RelayConnManager::new(redis_conn.clone()))),
            redis_conn,
        })
    }

    pub fn init(&mut self) -> bool {
        true
    }
}