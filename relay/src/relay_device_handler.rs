use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{get_query_param, ok_resp_str_map, ok_resp_vec_str_map, resp_empty_str_map, resp_empty_vec_str_map, RespMsgPair, RespStringMap, RespVecStringMap, StringMap};
use crate::{gRelayConnMgr, gRelaySettings, gRoomMgr};
use crate::relay_context::RelayContext;
use crate::relay_errors::{get_err_pair, ERR_DEVICE_NOT_FOUND, ERR_PARAM_INVALID, ERR_ROOM_NOT_FOUND};
use crate::relay_message::{KEY_DEVICE_ID, KEY_DEVICE_LOCAL_IPS, KEY_DEVICE_W3C_IP, KEY_PAGE, KEY_PAGE_SIZE, KEY_RELAY_SERVER_IP, KEY_RELAY_SERVER_PORT};

// handler device; query devices
// /query/devices
pub async fn hd_query_devices(State(_context): State<Arc<Mutex<RelayContext>>>,
                              query: Query<HashMap<String, String>>,
                              ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    ) -> Json<RespVecStringMap>  {
    let connections = gRelayConnMgr.lock().await.get_connections().await;
    let mut r = Vec::new();
    for conn in connections {
        r.push(conn.lock().await.as_str_map());
    }
    Json(ok_resp_vec_str_map(r))
}

// handler device; query device
// /query/device
pub async fn hd_query_device(State(_context): State<Arc<Mutex<RelayContext>>>,
                             query: Query<HashMap<String, String>>,
                             ConnectInfo(_addr): ConnectInfo<SocketAddr>,)
    -> Json<RespStringMap> {
    let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
    let connection = gRelayConnMgr.lock().await.get_connection(device_id).await;
    if let Some(conn) = connection {
        let device_id = conn.lock().await.device_id.clone();
        let client_w3c_ip = conn.lock().await.client_w3c_host.clone();
        let mut client_local_ips = "".to_string();
        for info in conn.lock().await.client_net_info.clone() {
            client_local_ips.push_str(info.ip.as_str());
            client_local_ips.push_str(";");
        }
        let server_w3c_ip = gRelaySettings.lock().await.server_w3c_ip.clone();
        let server_working_port = gRelaySettings.lock().await.server_working_port as i32;
        
        let mut value = StringMap::new();
        value.insert(KEY_DEVICE_ID.to_string(), device_id.clone());
        value.insert(KEY_DEVICE_W3C_IP.to_string(), client_w3c_ip.clone());
        value.insert(KEY_DEVICE_LOCAL_IPS.to_string(), client_local_ips);
        value.insert(KEY_RELAY_SERVER_IP.to_string(), server_w3c_ip.clone());
        value.insert(KEY_RELAY_SERVER_PORT.to_string(), server_working_port.to_string());
        return Json(base::ok_resp(value));
    }
    Json(resp_empty_str_map(get_err_pair(ERR_DEVICE_NOT_FOUND)))
}