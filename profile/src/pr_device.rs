use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PrDevice {
    //
    pub device_id: String,
    //
    #[serde(default)]
    pub user_id: String,
    //
    pub seed: String,
    //
    pub created_timestamp: i64,
    //
    pub last_update_timestamp: i64,
    //
    pub random_pwd_md5: String,
    //
    pub safety_pwd_md5: String,
    //
    // reset per month
    #[serde(default)]
    pub used_time: i64,
    
    pub gen_random_pwd: String,
}

impl Default for PrDevice {
    fn default() -> Self {
        PrDevice {
            device_id: "".to_string(),
            user_id: "".to_string(),
            seed: "".to_string(),
            created_timestamp: 0,
            last_update_timestamp: 0,
            random_pwd_md5: "".to_string(),
            safety_pwd_md5: "".to_string(),
            used_time: 0,
            gen_random_pwd: "".to_string(),
        }
    }
}