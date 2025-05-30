use std::collections::HashMap;
use std::sync::{Arc};
use axum::body::Bytes;
use tokio::sync::{Mutex};
use crate::relay_conn::RelayConn;
use crate::relay_message::{KEY_CREATE_TIMESTAMP, KEY_DEVICE_ID, KEY_LAST_UPDATE_TIMESTAMP, KEY_REMOTE_DEVICE_ID, KEY_ROOM_ID};

pub struct RelayRoom {
    pub device_id: String,
    pub remote_device_id: String,
    pub room_id: String,
    pub create_timestamp: i64,
    pub last_update_timestamp: i64,
    pub relay_conns: HashMap<String, Arc<Mutex<RelayConn>>>
}

impl Default for RelayRoom {
    fn default() -> Self {
        RelayRoom {
            device_id: "".to_string(),
            remote_device_id: "".to_string(),
            room_id: "".to_string(),
            create_timestamp: 0,
            last_update_timestamp: 0,
            relay_conns: HashMap::new(),
        }
    }
}

impl RelayRoom {
    pub fn is_valid(&self) -> bool {
        !self.relay_conns.is_empty()
            && !self.remote_device_id.is_empty()
            && !self.room_id.is_empty()
    }
    
    pub fn as_str_map(&self) -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert(KEY_DEVICE_ID.to_string(), self.device_id.clone());
        hm.insert(KEY_REMOTE_DEVICE_ID.to_string(), self.remote_device_id.clone());
        hm.insert(KEY_ROOM_ID.to_string(), self.room_id.clone());
        hm.insert(KEY_CREATE_TIMESTAMP.to_string(), self.create_timestamp.to_string());
        hm.insert(KEY_LAST_UPDATE_TIMESTAMP.to_string(), self.last_update_timestamp.to_string());
        hm
    }
    
    pub async fn notify_except(&self, except_id: String, relay_msg_index: i64, m: Bytes) {
        let mut conns = Vec::new();
        for (key, value) in self.relay_conns.clone() {
            if key != except_id {
                conns.push(value.clone());
            }
        }
        
        for conn in conns {
            let m = m.clone();
            let device_id = conn.lock().await.device_id.clone();
            let r = conn.lock().await.send_bin_message_with_index(relay_msg_index, m).await;
            if !r {
                tracing::warn!("notify to this device failed: {}", device_id)
            }
        }
    }

    // pub async fn notify_target(&self, target_id: &String, relay_msg_index: i64, m: Bytes) {
    //     tracing::info!("notify_target notify_except: {:?}, have {} devices", target_id, self.devices.len());
    //     for device in self.devices.values() {
    //         let device_id = device.lock().await.device_id.clone();
    //         if *device_id == *target_id {
    //             tracing::info!("relay to: {}", device_id);
    //             let device = device.clone();
    //             let m = m.clone();
    //             let r = device.lock().await.send_bin_message_with_index(relay_msg_index, m).await;
    //             if !r {
    //                 tracing::warn!("notify to this device failed: {}", device_id)
    //             }
    //             break;
    //         }
    //
    //     }
    // }
}