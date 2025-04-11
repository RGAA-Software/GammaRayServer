use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{RespMessage, StringMap};
use protocol::grpc_relay::RelayQueryDeviceRequest;
use crate::{gSpvrGrpcProfileClientMgr, gSpvrGrpcRelayClientMgr};
use crate::spvr_context::SpvrContext;
use crate::spvr_defs::{SpvrGrpcDeviceInfo, KEY_DEVICE_ID, KEY_DEVICE_LOCAL_IPS, KEY_DEVICE_W3C_IP, KEY_RELAY_SERVER_IP, KEY_RELAY_SERVER_PORT};
use crate::spvr_errors::{get_err_pair, ERR_DEVICE_NOT_FOUND, ERR_PARAM_INVALID};

// verify device info
pub async fn hd_verify_device_info_in_profile_server(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<StringMap>>  {
    let result: StringMap = Default::default();

    let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
    let device_password = query.get("device_password").unwrap_or(&"".to_string()).clone();
    if device_id.is_empty() || device_password.is_empty() {
        return Json(base::resp_empty_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }

    let grpc_clients
        = gSpvrGrpcProfileClientMgr.lock().await.get_connected_clients().await;

    // let mut target_device = Arc::new(None);

    for client in grpc_clients {
        // if let Some(ch) = &mut *client.lock().await.client.lock().await {
        //     let r = ch.query_device(RelayQueryDeviceRequest {
        //         device_id: device_id.clone(),
        //     }).await;
        //
        //     if let Err(e) = r {
        //         tracing::error!("Failed to query device: {:?}", e);
        //         continue;
        //     }
        //     let r = r.unwrap();
        //     let reply = r.into_inner();
        //     if reply.has_device {
        //         tracing::info!("Found device : {:?}", reply);
        //         target_device = Arc::new(Some(SpvrGrpcDeviceInfo {
        //             device_id,
        //             client_w3c_ip: reply.client_w3c_ip,
        //             client_local_ips: reply.client_local_ips,
        //             server_w3c_ip: reply.server_w3c_ip,
        //             server_working_port: reply.server_working_port,
        //         }));
        //         break;
        //     }
        // }
    }

    Json(base::resp_empty_str_map(get_err_pair(ERR_DEVICE_NOT_FOUND)))
}