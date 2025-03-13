use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::spvr_conn::SpvrConn;

pub struct SpvrConnManager {
    conns: Mutex<HashMap<String, Arc<Mutex<SpvrConn>>>>,
}

impl SpvrConnManager {
    pub fn new() -> Self {
        Self {
            conns: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_conn(&mut self, id: String, conn: Arc<Mutex<SpvrConn>>) {
        self.conns.lock().await.insert(id, conn);
    }
    
    pub async fn remove_conn(&self, id: String) {
        self.conns.lock().await.remove(id.as_str());
    }
}