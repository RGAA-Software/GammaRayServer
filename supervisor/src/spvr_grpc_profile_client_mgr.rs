use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_grpc_profile_client::SpvrGrpcProfileClient;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SpvrGrpcProfileClientMgr {
    profile_conns: Arc<Mutex<HashMap<String, Arc<Mutex<SpvrGrpcProfileClient>>>>>,
}

impl SpvrGrpcProfileClientMgr {
    pub fn new() -> Self {
        Self {
            profile_conns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_connected_clients(&self) -> Vec<Arc<Mutex<SpvrGrpcProfileClient>>> {
        let mut clients: Vec<Arc<Mutex<SpvrGrpcProfileClient>>> = Vec::new();
        for c in self.profile_conns.lock().await.values() {
            let c = c.clone();
            clients.push(c);
        }
        clients
    }
}

impl SpvrGrpcClientManager for SpvrGrpcProfileClientMgr {

    async fn on_hello(&self, server_id: String, msg: SpvrInnerHello) {
        
    }

    async fn on_heartbeat(&self, server_id: String, msg: SpvrInnerHeartBeat) {

    }

    async fn on_close(&self, server_id: String) {

    }
}