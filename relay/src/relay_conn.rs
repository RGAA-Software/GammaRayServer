use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use prost::bytes::BytesMut;
use prost::Message as ProstMessage;
use redis::AsyncCommands;
use tokio::sync::Mutex;
use protocol::relay::RelayMessage;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_context::RelayContext;
use crate::relay_proto_maker::make_error_message;
use crate::relay_room_mgr::RelayRoomManager;

pub struct RelayConn {
    pub context: Arc<Mutex<RelayContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub conn_mgr: Arc<Mutex<RelayConnManager>>,
    pub room_mgr: Arc<Mutex<RelayRoomManager>>,
    pub device_id: String,
    pub last_update_timestamp: i64,
    pub heartbeat_index: i64,
}

impl RelayConn {
    pub async fn new(context: Arc<Mutex<RelayContext>>,
               sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
               device_id: String) -> Arc<Mutex<RelayConn>> {
        let conn_mgr = context.lock().await.conn_mgr.clone();
        let room_mgr = context.lock().await.room_mgr.clone();
        Arc::new(Mutex::new(RelayConn {
            context,
            sender,
            conn_mgr,
            room_mgr,
            device_id,
            last_update_timestamp: base::get_current_timestamp(),
            heartbeat_index: 0,
        }))
    }

    pub async fn append_received_data_size(&mut self, size: usize) {
        // to redis; key: year/month/
        let key = format!("{}", "".to_string());
        let r = self.context.lock().await
            .redis_conn.lock().await
            .set::<String, String, ()>("".to_string(), "".to_string()).await;
        if let Err(e) = r {
            tracing::error!("update received data for: {} failed", key);
        }
    }

    pub async fn append_send_data_size(&mut self, size: usize) {
        // to redis
        let key = format!("{}", "".to_string());
        let r = self.context.lock().await
            .redis_conn.lock().await
            .set::<String, String, ()>("".to_string(), "".to_string()).await;
        if let Err(e) = r {
            tracing::error!("update send data for: {} failed", key);
        }
    }

    pub async fn on_hello(&mut self, m: RelayMessage) {
        self.last_update_timestamp = base::get_current_timestamp();
        //tracing::info!("received hello message: {}", m.from_device_id);
    }

    pub async fn on_heartbeat(&mut self, m: RelayMessage) {
        self.last_update_timestamp = base::get_current_timestamp();
        self.heartbeat_index = m.heartbeat.unwrap().index;
        self.room_mgr.lock().await
            .on_heartbeat_for_my_room(m.from_device_id).await;
        //tracing::info!("received heartbeat message: {}", m.heartbeat.unwrap().index);
    }

    pub async fn on_error(&self, m: RelayMessage) {

    }
    
    // send back to this connection itself.
    pub async fn send_binary_message(&mut self, om: Bytes) -> bool {
        if !self.is_alive() {
            tracing::warn!("this device is not alive : {}", self.device_id);
        }
        let size = om.len();
        let r = self.sender.lock().await.send(Message::Binary(om)).await;
        if let Err(r) = r {
            tracing::error!("error sending relay message: {r}");
            return false;
        }
        self.append_send_data_size(size).await;
        true
    }

    pub fn is_alive(&self) -> bool {
        base::get_current_timestamp() - self.last_update_timestamp < 60*1000
    }
}