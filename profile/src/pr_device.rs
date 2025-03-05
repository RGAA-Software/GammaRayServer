use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PrDevice {
    pub device_id: String,
    pub seed: String,
    pub created_timestamp: i64,
    pub last_update_timestamp: i64,
    pub random_pwd: String,
    pub safety_pwd: String,
}

impl Default for PrDevice {
    fn default() -> Self {
        PrDevice {
            device_id: "".to_string(),
            seed: "".to_string(),
            created_timestamp: 0,
            last_update_timestamp: 0,
            random_pwd: "".to_string(),
            safety_pwd: "".to_string(),
        }
    }
}