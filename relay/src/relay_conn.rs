use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use prost::bytes::BytesMut;
use prost::Message as ProstMessage;
use redis::AsyncCommands;
use tokio::sync::Mutex;
use crate::proto::tc::{RelayCreateRoomRespMessage, RelayErrorCode, RelayMessage, RelayMessageType, RelayRoomPreparedMessage};
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

    // pub async fn on_create_room(&mut self, m: RelayMessage, om: Bytes) {
    //     let sub = m.create_room.unwrap();
    //     let room = self.room_mgr.lock().await
    //         .create_room(sub.device_id.clone(), sub.remote_device_id.clone()).await;
    //     if let Some(room) = room {
    //         tracing::info!("created room: {}", room.room_id);
    //         let mut rl_msg = RelayMessage::default();
    //         rl_msg.set_type(RelayMessageType::KRelayCreateRoomResp);
    //         rl_msg.create_room_resp = Some(RelayCreateRoomRespMessage {
    //             device_id: sub.device_id,
    //             remote_device_id: sub.remote_device_id,
    //             room_id: room.room_id,
    //         });
    //
    //         let r = rl_msg.encode_to_vec();
    //         self.send_binary_message(Bytes::from(r)).await;
    //     }
    //     else {
    //         let r = make_error_message(RelayErrorCode::KRelayCodeCreateRoomFailed);
    //         self.send_binary_message(Bytes::from(r)).await;
    //     }
    // }

    // pub async fn on_request_control(&mut self, m: RelayMessage, om: Bytes) {
    //     let sub = m.request_control.unwrap();
    //     let remote_device_id = sub.remote_device_id;
    //     let remote_conn = self.conn_mgr.lock().await.get_connection(&remote_device_id).await;
    //     if let Some(remote_conn) = remote_conn {
    //         remote_conn.lock().await.send_binary_message(om).await;
    //         tracing::info!("request control message to: {}", remote_device_id);
    //     }
    //     else {
    //         let r = make_error_message(RelayErrorCode::KRelayCodeRemoteClientNotFound);
    //         self.send_binary_message(Bytes::from(r)).await;
    //     }
    // }

    // pub async fn on_request_control_resp(&self, m: RelayMessage, om: Bytes) {
    //     let sub = m.request_control_resp.unwrap();
    //     let req_device_id = sub.device_id.clone();
    //     let remote_device_id = sub.remote_device_id.clone();
    //     let req_device = self.conn_mgr.lock().await.get_connection(&req_device_id).await;
    //     if let None = req_device {
    //         tracing::error!("can't find device: {}", req_device_id);
    //         return;
    //     }
    //     let req_device = req_device.unwrap();
    //     req_device.lock().await.send_binary_message(om.clone()).await;
    //
    //     if sub.under_control {
    //         tracing::info!("{} is under control", sub.remote_device_id);
    //         let room_id = sub.room_id;
    //         let room = self.room_mgr.lock().await.find_room(&room_id).await;
    //         if let None = room {
    //             tracing::error!("can't find room: {}", room_id);
    //             return;
    //         }
    //         let room = room.unwrap();
    //
    //         let resp_device = self.conn_mgr.lock().await.get_connection(&sub.remote_device_id).await;
    //         if let None = resp_device {
    //             tracing::error!("can't find remote device: {}", sub.remote_device_id);
    //             return;
    //         }
    //         let resp_device = resp_device.unwrap();
    //
    //         let mut rl_msg = RelayMessage::default();
    //         rl_msg.set_type(RelayMessageType::KRelayRoomPrepared);
    //         rl_msg.room_prepared = Some(RelayRoomPreparedMessage {
    //             room_id,
    //             device_id: req_device_id,
    //             remote_device_id: sub.remote_device_id,
    //         });
    //         let r = rl_msg.encode_to_vec();
    //         let rr = r.clone();
    //
    //         // 1. to requester
    //         tokio::spawn(async move {
    //             req_device.lock().await.send_binary_message(Bytes::from(r)).await;
    //             tracing::info!("send prepared to device: {}", sub.device_id);
    //         });
    //
    //         // 2. to remote
    //         tokio::spawn(async move {
    //             tracing::info!("before send prepared to remote : {}", remote_device_id);
    //             resp_device.lock().await.send_binary_message(Bytes::from(rr)).await;
    //             tracing::info!("send prepared to remote : {}", remote_device_id);
    //         });
    //     }
    // }

    // pub async fn on_relay(&mut self, m: RelayMessage, om: Bytes) {
    //     // append received data size
    //     self.append_received_data_size(om.len()).await;
    //
    //     let sub = m.relay.unwrap();
    //     let from_device_id = m.from_device_id;
    //     for room_id in sub.room_ids.iter() {
    //         let room = self.room_mgr.lock().await.find_room(room_id).await;
    //         if let Some(room) = room {
    //             let from_device_id = from_device_id.clone();
    //             let om = om.clone();
    //             tokio::spawn(async move {
    //                 room.notify_except(&from_device_id, om).await;
    //             });
    //         }
    //     }
    // }

    // send back to this connection itself.
    pub async fn send_binary_message(&mut self, om: Bytes) -> bool {
        if !self.is_alive() {
            tracing::warn!("this device is not alive...")
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