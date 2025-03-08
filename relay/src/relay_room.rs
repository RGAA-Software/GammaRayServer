use std::collections::HashMap;
use std::sync::{Arc};
use axum::body::Bytes;
use tokio::sync::{Mutex};
use crate::relay_conn::RelayConn;

pub struct RelayRoom {
    pub device_id: String,
    pub remote_device_id: String,
    pub room_id: String,
    pub create_timestamp: i64,
    pub last_update_timestamp: i64,
    pub devices: HashMap<String, Arc<Mutex<RelayConn>>>
}

impl Default for RelayRoom {
    fn default() -> Self {
        RelayRoom {
            device_id: "".to_string(),
            remote_device_id: "".to_string(),
            room_id: "".to_string(),
            create_timestamp: 0,
            last_update_timestamp: 0,
            devices: HashMap::new(),
        }
    }
}

impl RelayRoom {
    pub fn is_valid(&self) -> bool {
        !self.devices.is_empty()
            && !self.remote_device_id.is_empty()
            && !self.room_id.is_empty()
    }
    
    pub async fn notify_except(&self, except_id: &String, m: Bytes) {
        tracing::info!("relay_room notify_except: {:?}, have {} devices", except_id, self.devices.len());
        for device in self.devices.values() {
            let device_id = device.lock().await.device_id.clone();
            if *device_id == *except_id {
                tracing::info!("ignore the device: {}", device_id);
                continue;
            }
            tracing::info!("relay to: {}", device_id);
            let device = device.clone();
            let m = m.clone();
            tokio::spawn(async move {
                let r = device.lock().await.send_binary_message(m).await;
                if !r {
                    tracing::warn!("notify to this device failed: {}", device_id)
                }
            });
        }
    }
}