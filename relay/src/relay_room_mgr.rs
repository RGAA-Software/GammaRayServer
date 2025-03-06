use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;

pub struct RelayRoomManager {
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RelayRoomManager {
    pub fn new(redis_conn: Arc<Mutex<MultiplexedConnection>>) -> Self {
        Self {
            redis_conn,   
        }
    }
}