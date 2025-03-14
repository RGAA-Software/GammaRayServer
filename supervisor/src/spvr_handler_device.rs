use std::default::Default;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{RespMessage, RespStringMap, StrMap, StringMap};
use protocol::grpc_relay::RelayQueryDeviceRequest;
use crate::{gSpvrConnMgr, gSpvrGrpcRelayClientMgr};
use crate::spvr_context::SpvrContext;
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_grpc_relay_client::SpvrGrpcRelayClient;

// handler device
pub async fn hd_get_device_info(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<StringMap>>  {
    let result: StringMap = Default::default();

    let grpc_clients
        = gSpvrGrpcRelayClientMgr.lock().await.get_connected_clients().await;

    for client in grpc_clients {
        if let Some(ch) = &mut *client.lock().await.client.lock().await {
            let r = ch.query_device(RelayQueryDeviceRequest {
                device_id: "".to_string(),
            }).await;

            if let Err(e) = r {
                tracing::error!("Failed to query device: {:?}", e);
                continue;
            }
            let r = r.unwrap();
            let reply = r.into_inner();
            tracing::info!("Device connected: {:?}", reply);
        }
    }

    Json(base::ok_resp(result))
}