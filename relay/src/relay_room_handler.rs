use crate::relay_context::RelayContext;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use base::{get_query_param, ok_resp_vec_str_map, resp_empty_vec_str_map, RespMsgPair, RespStringMap, RespVecStringMap};
use base::StringMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::relay_errors::{get_err_pair, ERR_PARAM_INVALID, ERR_ROOM_NOT_FOUND};
use crate::relay_message::{KEY_PAGE, KEY_PAGE_SIZE, KEY_ROOM_ID};

// handler room; query single room
pub async fn hr_query_room(
    State(context): State<Arc<Mutex<RelayContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespStringMap>  {
    let room_id = get_query_param(&query.0, KEY_ROOM_ID);
    if let None = room_id {
        return Json(base::resp_empty_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }
    
    let room_id = room_id.unwrap();
    let room = context.lock().await.
        room_mgr.lock().await.find_room(&room_id).await;
    if let None = room {
        return Json(base::resp_empty_str_map(get_err_pair(ERR_ROOM_NOT_FOUND)));
    }
    
    let room = room.unwrap();
    Json(base::ok_resp_str_map(room.as_str_map()))
}

// handler room; query rooms
pub async fn hr_query_rooms(
    State(context): State<Arc<Mutex<RelayContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespVecStringMap>  {
    let page = get_query_param(&query.0, KEY_PAGE);
    let page_size = get_query_param(&query.0, KEY_PAGE_SIZE);
    if let None = page {
        return Json(resp_empty_vec_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }
    if let None = page_size {
        return Json(resp_empty_vec_str_map(get_err_pair(ERR_PARAM_INVALID)));
    }
    let page = page.unwrap().parse::<i32>().unwrap();
    let page_size = page_size.unwrap().parse::<i32>().unwrap();
    
    let room_ids = context.lock().await
        .room_mgr.lock().await
        .find_room_ids(page, page_size).await;
    let mut r = Vec::new();
    for room_id in room_ids {
        let room = context.lock().await
            .room_mgr.lock().await.find_room(&room_id).await;
        if let Some(room) = room {
            r.push(room.as_str_map());
        }
    }
    Json(ok_resp_vec_str_map(r))
}