use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{get_query_param, ok_resp_vec_str_map, resp_empty_vec_str_map, RespVecStringMap};
use crate::{gRelayConnMgr, gRoomMgr};
use crate::relay_context::RelayContext;
use crate::relay_errors::{get_err_pair, ERR_PARAM_INVALID, ERR_ROOM_NOT_FOUND};
use crate::relay_message::{KEY_PAGE, KEY_PAGE_SIZE};

// handler device; query devices
pub async fn hd_query_devices(
    State(_context): State<Arc<Mutex<RelayContext>>>,
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