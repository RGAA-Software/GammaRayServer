use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello};
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_grpc_relay_client::SpvrGrpcRelayClient;

pub struct SpvrGrpcRelayClientManager {
    relay_clients: Arc<Mutex<HashMap<String, Arc<Mutex<SpvrGrpcRelayClient>>>>>,
}

impl SpvrGrpcRelayClientManager {
    pub fn new() -> Self {
        Self {
            relay_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_connected_clients(&self) -> Vec<Arc<Mutex<SpvrGrpcRelayClient>>> {
        let mut clients: Vec<Arc<Mutex<SpvrGrpcRelayClient>>> = Vec::new();
        for c in self.relay_clients.lock().await.values() {
            let c = c.clone();
            clients.push(c);
        }
        clients
    }
}

impl SpvrGrpcClientManager for SpvrGrpcRelayClientManager {

    async fn on_hello(&self, server_id: String, msg: SpvrInnerHello) {
        let conns = self.relay_clients.clone();
        tokio::spawn(async move {
            let relay_client = Arc::new(Mutex::new(SpvrGrpcRelayClient::new()));
            relay_client.lock().await.connect(msg.server_local_ip.clone(), msg.server_grpc_port as u16).await;
            SpvrGrpcRelayClient::guard(relay_client.clone()).await;
            conns.lock().await.insert(server_id.clone(), relay_client);
            tracing::info!("add RelayServer: {}", server_id);
        });
    }

    async fn on_heartbeat(&self, server_id: String, msg: SpvrInnerHeartBeat) {

    }

    async fn on_close(&self, server_id: String) {
        self.relay_clients.lock().await.remove(&server_id);
    }
}
