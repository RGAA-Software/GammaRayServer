use std::cmp::max;
use std::collections::HashMap;
use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Commands, RedisResult};
use tokio::sync::Mutex;
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_message::{KEY_CREATE_TIMESTAMP, KEY_DEVICE_ID, KEY_LAST_UPDATE_TIMESTAMP, KEY_REMOTE_DEVICE_ID, KEY_ROOM_ID};
use crate::relay_room::RelayRoom;

pub struct RelayRoomManager {
    pub redis_conn: Arc<Mutex<MultiplexedConnection>>,
    pub conn_mgr: Arc<Mutex<RelayConnManager>>,
}

impl RelayRoomManager {
    pub fn new(redis_conn: Arc<Mutex<MultiplexedConnection>>,
               conn_mgr: Arc<Mutex<RelayConnManager>>)
        -> Self {
        Self {
            redis_conn,
            conn_mgr,
        }
    }

    pub async fn create_room(&self, device_id: String, remote_device_id: String) -> Option<RelayRoom> {
        let conn_device =
            if let Some(device) = self.conn_mgr.lock().await.get_connection(&device_id).await {
            device
        } else {
            tracing::error!("Could not find device {}", device_id);
            return None;
        };

        let conn_remote_device =
            if let Some(remote_device) = self.conn_mgr.lock().await.get_connection(&remote_device_id).await {
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
            devices,
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
            .lock().await
            .hset_multiple::<String, &str, String, ()>(room_id.clone(), &relay_room_info).await;
        if let Err(err) = result {
            tracing::error!("insert to redis failed {:?}, room id: {}", err, room_id);
            return None
        }

        Some(relay_room)
    }

    pub async fn find_room(&self, room_id: &String) -> Option<RelayRoom> {
        let result = self.redis_conn.lock().await
            .hgetall::<&String, Vec<(String, String)>>(room_id).await;
        if let Err(err) = result {
            tracing::error!("Could not find room: {} in redis", room_id);
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

        if let Some(device) =  self.conn_mgr.lock().await.get_connection(&relay_room.device_id).await {
            relay_room.devices.insert(relay_room.device_id.clone(), device.clone());
            //tracing::info!("found device {:?}", relay_room.device_id);
        }
        if let Some(remote_device) = self.conn_mgr.lock().await.get_connection(&relay_room.remote_device_id).await {
            relay_room.devices.insert(relay_room.remote_device_id.clone(), remote_device.clone());
            //tracing::info!("found remote device {:?}", relay_room.remote_device_id);
        }

        if relay_room.is_valid() {
            Some(relay_room)
        } else {
            None
        }
    }

    pub async fn find_rooms(&self, page: i32, page_size: i32) -> Vec<RelayRoom> {
        let begin = max(0, page - 1) * page_size;

        let conn = &mut *self.redis_conn.lock().await;
        let pattern = "relay-room:*";
        let max_keys = 10;
        let cursor = begin as u64;
        let r: RedisResult<Vec<String>> = redis::cmd("SCAN")
            .cursor_arg(cursor)
            .arg("MATCH").arg(pattern)
            .arg("COUNT").arg(page_size)
            .query_async(conn)
            .await;
        if let Err(err) = r {
            return Vec::new()
        }
        let room_ids = r.unwrap();
        room_ids.iter().for_each(|room_id| {
            tracing::info!("Found room {}", room_id);
        });

        Vec::new()
    }

    pub async fn destroy_room_by_creator(&self, device_id: String) {
        let r = self.redis_conn.lock().await
            .keys::<String, Vec<String>>(format!("relay-room:{}*", device_id)).await;
        if let Err(err) = r {
            tracing::error!("Could not find rooms created by: {}, err: {}", device_id, err);
            return;
        }

        let room_ids = r.unwrap();
        for room_id in room_ids.iter() {
            let room_info =
                self.redis_conn.lock().await.hgetall::<&String, Vec<(String, String)>>(room_id).await;
            if let Err(err) = room_info {
                tracing::error!("Could not find room info: {} in redis", room_id);
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
                // todo: Notify it, room has been destroyed.
            }

            //
            _ = self.redis_conn.lock().await.del::<&String, ()>(room_id).await;
        }
    }

    pub async fn on_heartbeat_for_my_room(&self, device_id: String) {
        let r = self.redis_conn.lock().await
            .keys::<String, Vec<String>>(format!("relay-room:{}*", device_id)).await;
        if let Err(err) = r {
            tracing::error!("Could not find rooms created by: {}, err: {}", device_id, err);
            return;
        }

        let room_ids = r.unwrap();
        for room_id in room_ids.iter() {
            _ = self.redis_conn.lock().await
                .hset::<&String, &str, String, ()>(room_id, KEY_LAST_UPDATE_TIMESTAMP, base::get_current_timestamp().to_string()).await;
        }
    }
}