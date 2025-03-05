use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PrDevice {
    pub device_id: String,
    pub created_timestamp: i64,
    pub last_update_timestamp: i64,
}

impl Default for PrDevice {
    fn default() -> Self {
        PrDevice {
            device_id: "".to_string(),
            created_timestamp: 0,
            last_update_timestamp: 0,
        }
    }
}