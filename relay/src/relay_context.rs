use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_device_mgr::RelayDeviceManager;
use crate::relay_room::RelayRoom;
use crate::relay_room_mgr::RelayRoomManager;

pub struct RelayContext {
    pub conn_mgr: Arc<Mutex<RelayConnManager>>,
    pub room_mgr: Arc<Mutex<RelayRoomManager>>,
    pub device_mgr: Arc<Mutex<RelayDeviceManager>>,
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

        let conn_mgr = Arc::new(Mutex::new(RelayConnManager::new()));

        let room_mgr = RelayRoomManager::new(redis_conn.clone(), conn_mgr.clone());
        let room_mgr = Arc::new(Mutex::new(room_mgr));

        Ok(RelayContext {
            conn_mgr,
            room_mgr,
            device_mgr: Arc::new(Mutex::new(RelayDeviceManager::new())),
            redis_conn,
        })
    }

    pub fn init(&mut self) -> bool {
        true
    }
}