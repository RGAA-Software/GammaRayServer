use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::dash_conn::DashConnPtr;

pub struct DashConnManager {
    conns: Mutex<HashMap<String, DashConnPtr>>,
}

impl DashConnManager {
    pub fn new() -> Self {
        DashConnManager {
            conns: Default::default(),
        }
    }

    pub async fn add_conn(&mut self, id: String, conn: DashConnPtr) {
        self.conns.lock().await.insert(id, conn);
    }

    pub async fn remove_conn(&self, id: String) {
        self.conns.lock().await.remove(id.as_str());
    }

    pub async fn get_all_conns(&self) -> HashMap<String, DashConnPtr> {
        self.conns.lock().await.clone()
    }
}