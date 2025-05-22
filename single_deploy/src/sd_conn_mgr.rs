use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::sd_conn::SdConnPtr;

pub struct SdConnManager {
    conns: Mutex<HashMap<String, SdConnPtr>>,
}

impl SdConnManager {
    pub fn new() -> Self {
        Self {
            conns: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_conn(&mut self, id: String, conn: SdConnPtr) {
        self.conns.lock().await.insert(id, conn);
    }

    pub async fn remove_conn(&self, id: String) {
        self.conns.lock().await.remove(id.as_str());
    }

    pub async fn get_all_conns(&self) -> HashMap<String, SdConnPtr> {
        self.conns.lock().await.clone()
    }
}