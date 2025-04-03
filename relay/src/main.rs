mod relay_context;
mod relay_server;
mod relay_conn_mgr;
mod relay_conn;
mod relay_room_mgr;
mod relay_message;
mod relay_room;
mod relay_proto_maker;
mod relay_room_handler;
mod relay_errors;
mod relay_settings;
mod relay_statistics;
mod relay_grpc_server;
mod relay_spvr_client;
mod relay_queue;

use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, filter::LevelFilter};
use tracing_subscriber::prelude::*;
use base::log_util;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_context::RelayContext;
use crate::relay_grpc_server::RelayGrpcServer;
use crate::relay_room_mgr::RelayRoomManager;
use crate::relay_server::RelayServer;
use crate::relay_settings::RelaySettings;
use crate::relay_spvr_client::RelaySpvrClient;

lazy_static::lazy_static! {
    pub static ref gRelaySettings: Arc<Mutex<RelaySettings>> = Arc::new(Mutex::new(RelaySettings::new()));
    pub static ref gRelayGrpcServer: Arc<Mutex<RelayGrpcServer>> = Arc::new(Mutex::new(RelayGrpcServer::new()));
    pub static ref gRelaySpvrClient: Arc<Mutex<RelaySpvrClient>> = Arc::new(Mutex::new(RelaySpvrClient::new()));
    pub static ref gRelayConnMgr: Arc<Mutex<RelayConnManager>> = Arc::new(Mutex::new(RelayConnManager::new()));
    pub static ref gRedisConn: Arc<Mutex<Option<MultiplexedConnection>>> = Arc::new(Mutex::new(None));
    pub static ref gRoomMgr: Arc<Mutex<Option<RelayRoomManager>>> = Arc::new(Mutex::new(None));
}

#[tokio::main]
async fn main() {
    let _guard = log_util::init_log("logs/relay/".to_string(), "log_relay".to_string());

    // settings
    gRelaySettings.lock().await.load().await;

    // redis
    {
        let redis_url = gRelaySettings.lock().await.redis_url.clone();
        let redis_client = redis::Client::open(redis_url.clone()).unwrap();
        let redis_conn = redis_client.get_multiplexed_async_connection().await;
        if let Err(err) = redis_conn {
            tracing::error!("connect to redis failed: {}", err.to_string());
            return;
        }
        tracing::info!("connect to redis: {}", redis_url);
        let redis_conn = redis_conn.unwrap();
        // redis_guard will hold the MutexGuard<>, MUST use a temporary variable
        let mut redis_guard = gRedisConn.lock().await;
        *redis_guard = Some(redis_conn);
    }

    // room manager
    {
        // room_mgr_guard will hold the MutexGuard<>, MUST use a temporary variable
        let mut room_mgr_guard = gRoomMgr.lock().await;
        *room_mgr_guard = Some(RelayRoomManager::new(gRedisConn.clone(), gRelayConnMgr.clone()));
    }

    let context = RelayContext::new();
    let context = Arc::new(Mutex::new(context));
    context.lock().await.init();

    tokio::spawn(async move {
        tracing::info!("will start grpc relay server.");
        let grpc_relay = RelayGrpcServer::new();
        grpc_relay.start().await;
        tracing::info!("after grpc relay server.");
    });

    let spvr_srv_ip = gRelaySettings.lock().await.spvr_server_ip.clone();
    let spvr_srv_port = gRelaySettings.lock().await.spvr_server_port;
    let srv_id = gRelaySettings.lock().await.server_id.clone();
    let address = format!("ws://{}:{}/inner?server_id={}&server_type=0", spvr_srv_ip, spvr_srv_port, srv_id);
    tracing::info!("connecting to: {}", address);
    gRelaySpvrClient.lock().await.connect(address).await;

    let server = RelayServer::new("0.0.0.0".to_string(),
                                  gRelaySettings.lock().await.server_working_port,
                                  context.clone());
    server.start().await;
}
