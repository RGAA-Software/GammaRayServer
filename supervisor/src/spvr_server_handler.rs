use crate::spvr_context::SpvrContext;
use crate::gSpvrConnMgr;
use axum::extract::{ConnectInfo, Query, State};
use axum::Json;
use base::{RespMessage, StrMap};
use std::collections::HashMap;
use std::default::Default;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

// handler server
pub async fn hs_get_online_servers(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<Vec<StrMap>>>  {
    let mut result: Vec<StrMap> = Default::default();
    
    // 1. profile server
    let profile_conns = gSpvrConnMgr.lock().await.get_profile_conns().await;
    for conn in profile_conns {
        result.push(conn.lock().await.get_conn_server_info().await);
    }

    // 2. relay server
    let relay_conns = gSpvrConnMgr.lock().await.get_relay_conns().await;
    for conn in relay_conns {
        result.push(conn.lock().await.get_conn_server_info().await);
    }

    Json(base::ok_resp(result))
}

pub async fn hs_get_online_profile_servers(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<Vec<StrMap>>> {

    let mut result: Vec<StrMap> = Default::default();
    let profile_conns = gSpvrConnMgr.lock().await.get_profile_conns().await;
    for conn in profile_conns {
        result.push(conn.lock().await.get_conn_server_info().await);
    }
    Json(base::ok_resp(result))
}

pub async fn hs_get_online_relay_servers(
    State(context): State<Arc<Mutex<SpvrContext>>>,
    query: Query<HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<RespMessage<Vec<StrMap>>> {
    
    let mut result: Vec<StrMap> = Default::default();
    let relay_conns = gSpvrConnMgr.lock().await.get_relay_conns().await;
    for conn in relay_conns {
        result.push(conn.lock().await.get_conn_server_info().await);
    }
    Json(base::ok_resp(result))
}