use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::Mutex;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello};
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_grpc_profile_client::SpvrGrpcProfileClient;

pub struct SpvrGrpcProfileClientMgr {
    profile_conns: Arc<Mutex<HashMap<String, Arc<Mutex<SpvrGrpcProfileClient>>>>>,
}

impl SpvrGrpcProfileClientMgr {
    pub fn new() -> Self {
        Self {
            profile_conns: Arc::new(Mutex::new(HashMap::new())),
        }
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