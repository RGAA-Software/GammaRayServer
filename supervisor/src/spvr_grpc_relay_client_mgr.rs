use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello, SpvrServerType};
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_grpc_profile_client::SpvrGrpcProfileClient;
use crate::spvr_grpc_relay_client::SpvrGrpcRelayClient;
use crate::spvr_server::SpvrServer;

pub struct SpvrGrpcRelayClientManager {
    relay_clients: Arc<Mutex<HashMap<String, Arc<Mutex<SpvrGrpcRelayClient>>>>>,
}

impl SpvrGrpcRelayClientManager {
    pub fn new() -> Self {
        Self {
            relay_clients: Arc::new(Mutex::new(HashMap::new())),
        }
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
        let conns = self.relay_clients.clone();
        tokio::spawn(async move {
            conns.lock().await.remove(&server_id);
            tracing::info!("remove RelayServer: {}", server_id);
        });
    }
}
