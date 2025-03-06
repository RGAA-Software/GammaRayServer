use std::sync::Arc;
use tokio::sync::Mutex;
use crate::relay_conn_mgr::RelayConnManager;

pub struct RelayContext {
    pub device_conn_mgr: Arc<Mutex<RelayConnManager>>,
}

impl RelayContext {
    pub fn new() -> Self {
        RelayContext {
            device_conn_mgr: Arc::new(Mutex::new(RelayConnManager::new())),
        }
    }

    pub fn init(&mut self) -> bool {
        true
    }
}