use std::default::Default;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{RespMessage, StrMap, StringMap};
use protocol::grpc_relay::RelayQueryDeviceRequest;
use crate::{gSpvrGrpcRelayClientMgr};
use crate::spvr_context::SpvrContext;
use crate::spvr_defs::{SpvrGrpcDeviceInfo, KEY_DEVICE_ID, KEY_DEVICE_LOCAL_IPS, KEY_DEVICE_W3C_IP, KEY_RELAY_SERVER_IP, KEY_RELAY_SERVER_PORT, KEY_W3C_IP};
use crate::spvr_errors::{get_err_pair, ERR_DEVICE_NOT_FOUND, ERR_PARAM_INVALID};

// handler device
// get device info from RelayServer
pub async fn hd_get_device_info_from_relay_server(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<StringMap>>  {
    let result: StringMap = Default::default();

    let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
    if device_id.is_empty() {
        return Json(base::resp_empty_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }

    let grpc_clients
        = gSpvrGrpcRelayClientMgr.lock().await.get_connected_clients().await;

    let mut target_device = Arc::new(None);

    for client in grpc_clients {
        if let Some(ch) = &mut *client.lock().await.client.lock().await {
            let r = ch.query_device(RelayQueryDeviceRequest {
                device_id: device_id.clone(),
            }).await;

            if let Err(e) = r {
                tracing::error!("Failed to query device: {:?}", e);
                continue;
            }
            let r = r.unwrap();
            let reply = r.into_inner();
            if reply.has_device {
                tracing::info!("Found device : {:?}", reply);
                target_device = Arc::new(Some(SpvrGrpcDeviceInfo {
                    device_id,
                    client_w3c_ip: reply.client_w3c_ip,
                    client_local_ips: reply.client_local_ips,
                    server_w3c_ip: reply.server_w3c_ip,
                    server_working_port: reply.server_working_port,
                }));
                break;
            }
        }
    }

    if let Some(target_device) = & *target_device {
        let local_ips = target_device.client_local_ips.clone();
        let mut device_local_ips = "".to_string();
        for ip in local_ips {
            device_local_ips.push_str(&ip);
            device_local_ips.push_str(";");
        }
        let mut value = StringMap::new();
        value.insert(KEY_DEVICE_ID.to_string(), target_device.device_id.clone());
        value.insert(KEY_DEVICE_W3C_IP.to_string(), target_device.client_w3c_ip.clone());
        value.insert(KEY_DEVICE_LOCAL_IPS.to_string(), device_local_ips);
        value.insert(KEY_RELAY_SERVER_IP.to_string(), target_device.server_w3c_ip.clone());
        value.insert(KEY_RELAY_SERVER_PORT.to_string(), target_device.server_working_port.to_string());
        Json(base::ok_resp(value))
    }
    else {
        Json(base::resp_empty_str_map(get_err_pair(ERR_DEVICE_NOT_FOUND)))
    }
}