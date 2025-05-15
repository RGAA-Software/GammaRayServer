use std::cmp::max;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use axum::body::Bytes;
use futures_util::stream::SplitStream;
use prost::Message;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Commands, RedisResult};
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use protocol::relay::{RelayCreateRoomRespMessage, RelayErrorCode, RelayMessage, RelayMessageType, RelayRoomDestroyedMessage, RelayRoomPreparedMessage};
use crate::gRedisConn;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_message::{KEY_CREATE_TIMESTAMP, KEY_DEVICE_ID, KEY_LAST_UPDATE_TIMESTAMP, KEY_REMOTE_DEVICE_ID, KEY_ROOM_ID};
use crate::relay_proto_maker::make_error_message;
use crate::relay_queue::{RelayPacket, RelayQueue};
use crate::relay_room::RelayRoom;

pub struct RelayRoomManager {
    pub redis_conn: Arc<Mutex<Option<MultiplexedConnection>>>,
    pub conn_mgr: Arc<Mutex<RelayConnManager>>,
    pub relay_queue: Arc<Mutex<HashMap<String, RelayQueue>>>,

}

impl RelayRoomManager {
    pub fn new(redis_conn: Arc<Mutex<Option<MultiplexedConnection>>>,
               conn_mgr: Arc<Mutex<RelayConnManager>>)
        -> Self {
        Self {
            redis_conn,
            conn_mgr,
            relay_queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_room(&self, device_id: String, remote_device_id: String) -> Option<RelayRoom> {
        let conn_device =
            if let Some(device)
                = self.conn_mgr.lock().await.get_connection(device_id.clone()).await {
            device
        } else {
            tracing::error!("Could not find device {}", device_id);
            return None;
        };

        let conn_remote_device =
            if let Some(remote_device)
                = self.conn_mgr.lock().await.get_connection(remote_device_id.clone()).await {
            remote_device
        } else {
            tracing::error!("Could not find remote device {}", remote_device_id);
            return None;
        };

        let room_id = format!("relay-room:{}-{}", device_id, remote_device_id);
        let mut devices = HashMap::new();
        devices.insert(device_id.clone(), conn_device);
        devices.insert(remote_device_id.clone(), conn_remote_device);

        let relay_room = RelayRoom {
            device_id: device_id.clone(),
            remote_device_id: remote_device_id.clone(),
            room_id: room_id.clone(),
            create_timestamp: base::get_current_timestamp(),
            last_update_timestamp: base::get_current_timestamp(),
            relay_conns: devices,
        };

        // to redis
        let relay_room_info = [
            (KEY_DEVICE_ID, device_id),
            (KEY_REMOTE_DEVICE_ID, remote_device_id),
            (KEY_ROOM_ID, room_id.clone()),
            (KEY_CREATE_TIMESTAMP, relay_room.create_timestamp.to_string()),
            (KEY_LAST_UPDATE_TIMESTAMP, relay_room.last_update_timestamp.to_string()),
        ];

        let result = self.redis_conn
            .lock().await.as_mut().expect("")
            .hset_multiple::<String, &str, String, ()>(room_id.clone(), &relay_room_info).await;
        if let Err(err) = result {
            tracing::error!("insert to redis failed {:?}, room id: {}", err, room_id);
            return None
        }

        // relay queue
        let mut queue = RelayQueue::new(room_id.clone());
        queue.run().await;
        self.relay_queue.lock().await.insert(room_id.clone(), queue);

        Some(relay_room)
    }

    pub async fn find_room(&self, room_id: String) -> Option<RelayRoom> {
        let result = self.redis_conn.lock().await.as_mut().expect("")
            .hgetall::<String, Vec<(String, String)>>(room_id.clone()).await;
        if let Err(err) = result {
            tracing::error!("Could not find room: {} in redis, err: {}", room_id, err);
            return None
        }

        let mut relay_room = RelayRoom::default();
        let room_info = result.unwrap();
        for (key, val) in room_info.iter() {
            if key == KEY_DEVICE_ID {
                relay_room.device_id = val.clone();
            }
            else if key == KEY_REMOTE_DEVICE_ID {
                relay_room.remote_device_id = val.clone();
            }
            else if key == KEY_ROOM_ID {
                relay_room.room_id = val.clone();
            }
            else if key == KEY_CREATE_TIMESTAMP {
                relay_room.create_timestamp = val.parse::<i64>().unwrap_or(0);
            }
            else if key == KEY_LAST_UPDATE_TIMESTAMP {
                relay_room.last_update_timestamp = val.parse::<i64>().unwrap_or(0);
            }
        }

        if let Some(device) =  self.conn_mgr.lock().await.get_connection(relay_room.device_id.clone()).await {
            relay_room.relay_conns.insert(relay_room.device_id.clone(), device.clone());
            //tracing::info!("found device {:?}", relay_room.device_id);
        }
        if let Some(remote_device) = self.conn_mgr.lock().await.get_connection(relay_room.remote_device_id.clone()).await {
            relay_room.relay_conns.insert(relay_room.remote_device_id.clone(), remote_device.clone());
            //tracing::info!("found remote device {:?}", relay_room.remote_device_id);
        }

        if relay_room.is_valid() {
            Some(relay_room)
        } else {
            None
        }
    }

    pub async fn find_room_ids(&self, page: i32, page_size: i32) -> Vec<String> {
        let begin = max(0, page - 1) * page_size;
        let pattern = "relay-room:*";
        let cursor = begin as u64;
        let r: RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
            .cursor_arg(cursor)
            .arg("MATCH").arg(pattern)
            .arg("COUNT").arg(page_size)
            .query_async(self.redis_conn.lock().await.as_mut().expect(""))
            .await;
        if let Err(err) = r {
            tracing::error!("Could not find rooms in redis, err: {}", err);
            return Vec::new()
        }
        let room_ids = r.unwrap();
        tracing::info!("room ids: {:#?}", room_ids);
        room_ids.1
    }

    pub async fn destroy_room_by_creator(&self, device_id: String) {
        let r = self.redis_conn.lock().await.as_mut().expect("")
            .keys::<String, Vec<String>>(format!("relay-room:{}*", device_id)).await;
        if let Err(err) = r {
            tracing::error!("Could not find rooms created by: {}, err: {}", device_id, err);
            return;
        }

        let room_ids = r.unwrap();
        for room_id in room_ids.iter() {
            let room_info =
                self.redis_conn.lock().await.as_mut().expect("")
                    .hgetall::<&String, Vec<(String, String)>>(room_id).await;
            if let Err(err) = room_info {
                tracing::error!("Could not find room info: {} in redis, err: {}", room_id, err);
                continue;
            }

            let mut target_remote_device_id = "".to_string();
            let room_info = room_info.unwrap();
            for (key, val) in room_info.iter() {
                if key == KEY_REMOTE_DEVICE_ID {
                    target_remote_device_id = val.clone();
                    break;
                }
            }

            if !target_remote_device_id.is_empty() {
                let mut rl_msg = RelayMessage::default();
                rl_msg.set_type(RelayMessageType::KRelayRoomDestroyed);
                rl_msg.room_destroyed = Some(RelayRoomDestroyedMessage {
                    room_id: room_id.clone(),
                    device_id: device_id.clone(),
                    remote_device_id: target_remote_device_id.clone(),
                });
                let r = rl_msg.encode_to_vec();
                if let Some(remote_device) = self.conn_mgr.lock().await
                    .get_connection(target_remote_device_id).await {
                    tokio::spawn(async move {
                        _ = remote_device.lock().await.send_bin_message(Bytes::from(r)).await;
                    });
                }
            }

            // delete info in redis
            _ = self.redis_conn.lock().await.as_mut().expect("").del::<&String, ()>(room_id).await;

            // exit relay queue
            let relay_queue = self.relay_queue.clone();
            let rid = room_id.clone();
            tokio::spawn(async move {
                if let Some(queue) = relay_queue.lock().await.get(rid.as_str()) {
                    queue.exit().await;
                }
            });
        }
    }

    pub async fn on_heartbeat_for_my_room(&self, device_id: String) {
        let r = gRedisConn.lock().await
            .as_mut().expect("")
            .keys::<String, Vec<String>>(format!("relay-room:{}*", device_id)).await;
        let room_ids = r.unwrap();
        for room_id in room_ids.iter() {
            _ = gRedisConn.lock().await
                .as_mut().expect("")
                .hset::<&String, &str, String, ()>(room_id, KEY_LAST_UPDATE_TIMESTAMP, base::get_current_timestamp().to_string()).await;
        }
    }

    pub async fn on_relay(&mut self, m: RelayMessage, om: Bytes) {
        // append received data size
        //self.append_received_data_size(om.len()).await;

        let sub = m.relay.unwrap();
        let from_device_id = m.from_device_id;
        let relay_msg_index = sub.relay_msg_index;
        for room_id in sub.room_ids.iter() {
            let room = self.find_room(room_id.clone()).await;
            if let Some(room) = room {
                let from_device_id = from_device_id.clone();
                let om = om.clone();

                let id = room_id.clone();
                if let Some(queue) = self.relay_queue.lock().await.get(&id) {
                    queue.send(RelayPacket {
                        except_id: from_device_id,
                        room,
                        payload: om,
                        relay_msg_index,
                    }).await;
                }
            }
        }
    }

    pub async fn on_create_room(&self, m: RelayMessage, _om: Bytes) {
        let sub = m.create_room.unwrap();
        let room = self.create_room(sub.device_id.clone(), sub.remote_device_id.clone()).await;
        let resp_msg;
        if let Some(room) = room {
            tracing::info!("created room: {}", room.room_id);
            let mut rl_msg = RelayMessage::default();
            rl_msg.set_type(RelayMessageType::KRelayCreateRoomResp);
            rl_msg.create_room_resp = Some(RelayCreateRoomRespMessage {
                device_id: sub.device_id.clone(),
                remote_device_id: sub.remote_device_id,
                room_id: room.room_id.clone(),
            });

            resp_msg = rl_msg.encode_to_vec();
        }
        else {
            resp_msg = make_error_message(RelayErrorCode::KRelayCodeCreateRoomFailed);
        }

        if let Some(device) = self.conn_mgr.lock().await
            .get_connection(sub.device_id.clone()).await {
            _ = device.lock().await.send_bin_message(Bytes::from(resp_msg)).await;
        }
    }

    pub async fn on_request_control(&mut self, m: RelayMessage, om: Bytes) {
        let from_device_id = m.from_device_id;
        let sub = m.request_control.unwrap();
        let remote_device_id = sub.remote_device_id;
        let remote_conn
            = self.conn_mgr.lock().await.get_connection(remote_device_id.clone()).await;

        if let Some(remote_conn) = remote_conn {
            remote_conn.lock().await.send_bin_message(om).await;
            tracing::info!("request control message to: {}", remote_device_id);
        }
        else {
            if let Some(conn)
                = self.conn_mgr.lock().await.get_connection(from_device_id).await {
                let r = make_error_message(RelayErrorCode::KRelayCodeRemoteClientNotFound);
                _ = conn.lock().await.send_bin_message(Bytes::from(r)).await;
            }
        }
    }

    pub async fn on_request_control_resp(&self, m: RelayMessage, om: Bytes) {
        let sub = m.request_control_resp.unwrap();
        let req_device_id = sub.device_id.clone();
        //let remote_device_id = sub.remote_device_id.clone();
        let req_device
            = self.conn_mgr.lock().await.get_connection(req_device_id.clone()).await;
        if let None = req_device {
            tracing::error!("can't find device: {}", req_device_id);
            return;
        }
        let req_device = req_device.unwrap();
        req_device.lock().await.send_bin_message(om.clone()).await;

        if sub.under_control {
            tracing::info!("{} is under control", sub.remote_device_id);
            let room_id = sub.room_id;
            let room = self.find_room(room_id.clone()).await;
            if let None = room {
                tracing::error!("can't find room: {}", room_id);
                return;
            }
            //let room = room.unwrap();

            let resp_device =
                self.conn_mgr.lock().await.get_connection(sub.remote_device_id.clone()).await;
            if let None = resp_device {
                tracing::error!("can't find remote device: {}", sub.remote_device_id);
                return;
            }
            let resp_device = resp_device.unwrap();

            let mut rl_msg = RelayMessage::default();
            rl_msg.set_type(RelayMessageType::KRelayRoomPrepared);
            rl_msg.room_prepared = Some(RelayRoomPreparedMessage {
                room_id,
                device_id: req_device_id,
                remote_device_id: sub.remote_device_id,
            });
            let r = rl_msg.encode_to_vec();
            let rr = r.clone();

            // 1. to requester
            tokio::spawn(async move {
                req_device.lock().await.send_bin_message(Bytes::from(r)).await;
            });

            // 2. to remote
            tokio::spawn(async move {
                resp_device.lock().await.send_bin_message(Bytes::from(rr)).await;
            });
        }
    }

    pub async fn on_request_resume_pause_stream(&self, m: RelayMessage, om: Bytes) {
        let from_device_id = m.from_device_id;
        let mut remote_device_id = "".to_string();
        if m.r#type == RelayMessageType::KRelayRequestResumeStream {
            let sub = m.request_resume.unwrap();
            remote_device_id = sub.remote_device_id;    
        }
        else if m.r#type == RelayMessageType::KRelayRequestPausedStream {
            let sub = m.request_pause.unwrap();
            remote_device_id = sub.remote_device_id;
        }
        
        let remote_conn
            = self.conn_mgr.lock().await.get_connection(remote_device_id.clone()).await;

        if let Some(remote_conn) = remote_conn {
            remote_conn.lock().await.send_bin_message(om).await;
            tracing::info!("request pause stream message to: {}", remote_device_id);
        }
        else {
            if let Some(conn)
                = self.conn_mgr.lock().await.get_connection(from_device_id).await {
                let r = make_error_message(RelayErrorCode::KRelayCodeRemoteClientNotFound);
                _ = conn.lock().await.send_bin_message(Bytes::from(r)).await;
            }
        }
    }
}