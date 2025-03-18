use crate::spvr_conn::SpvrConnPtr;
use protocol::spvr_inner::SpvrServerType;
use std::collections::HashMap;
use tokio::sync::Mutex;

pub struct SpvrConnManager {
    conns: Mutex<HashMap<String, SpvrConnPtr>>,
}

impl SpvrConnManager {
    pub fn new() -> Self {
        Self {
            conns: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add_conn(&mut self, id: String, conn: SpvrConnPtr) {
        self.conns.lock().await.insert(id, conn);
    }
    
    pub async fn remove_conn(&self, id: String) {
        self.conns.lock().await.remove(id.as_str());
    }
    
    pub async fn get_all_conns(&self) -> HashMap<String, SpvrConnPtr> {
        self.conns.lock().await.clone()
    }
    
    pub async fn get_profile_conns(&self) -> Vec<SpvrConnPtr> {
        self.get_conn_by_type(SpvrServerType::KSpvrProfileServer).await
    }
    
    pub async fn get_relay_conns(&self) -> Vec<SpvrConnPtr> {
        self.get_conn_by_type(SpvrServerType::KSpvrRelayServer).await
    }
    
    async fn get_conn_by_type(&self, srv_type: SpvrServerType) -> Vec<SpvrConnPtr> {
        let mut result = Vec::new();
        for (_, conn) in self.conns.lock().await.iter() {
            if conn.lock().await.server_type == srv_type {
                result.push(conn.clone());
            }
        }
        result
    }
}