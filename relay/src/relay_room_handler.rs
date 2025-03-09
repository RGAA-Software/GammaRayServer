use crate::relay_context::RelayContext;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use base::{get_query_param, RespMsgPair, RespStringMap};
use base::StringMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::relay_errors::{get_err_pair, ERR_PARAM_INVALID, ERR_ROOM_NOT_FOUND};
use crate::relay_message::KEY_ROOM_ID;

// handler room
pub async fn hr_query_room(
    State(context): State<Arc<Mutex<RelayContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespStringMap>  {
    let room_id = get_query_param(&query.0, KEY_ROOM_ID);
    if let None = room_id {
        return Json(base::make_resp_empty_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }
    
    let room_id = room_id.unwrap();
    let room = context.lock().await.
        room_mgr.lock().await.find_room(&room_id).await;
    if let None = room {
        return Json(base::make_resp_empty_str_map(get_err_pair(ERR_ROOM_NOT_FOUND)));
    }
    
    let room = room.unwrap();
    Json(base::make_ok_resp_str_map(room.as_str_map()))
}