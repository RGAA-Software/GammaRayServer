use sysinfo::{
    Components, Disks, Networks, System,
};
use base::system_info;

pub struct RelaySettings {
    pub server_id: String,
}

impl RelaySettings {
    pub fn new() -> RelaySettings {
        RelaySettings::default()
    }

    pub async fn init(&mut self) {
        let system_info = system_info::SystemInfo::new();
        self.server_id = system_info.server_id.clone();
        tracing::info!("system info: {:?}", system_info);
        tracing::info!("server id: {:?}", self.server_id);
    }
}

impl Default for RelaySettings {
    fn default() -> Self {
        RelaySettings {
            server_id: "".to_string(),
        }
    }
}