use serde::Deserialize;
use sysinfo::{
    Components, Disks, Networks, System,
};
use base::system_info;

#[derive(Debug, Deserialize)]
pub struct RelaySettings {
    pub server_id: String,
    pub server_name: String,
    pub server_port: u16,
}

impl RelaySettings {
    pub fn new() -> RelaySettings {
        RelaySettings::default()
    }

    pub async fn load(&mut self) {
        let toml_content = std::fs::read_to_string("spvr_settings.toml")
            .expect("can't read spvr_settings.toml");
        let settings: RelaySettings = toml::from_str(&toml_content).expect("parse toml failed");
        self.copy_from(&settings);
        
        if self.server_id.is_empty() {
            let system_info = system_info::SystemInfo::new();
            self.server_id = system_info.server_id;
        }
        
        println!("{:#?}", self);
    }

    fn copy_from(&mut self, source: &RelaySettings) {
        self.server_id = source.server_id.clone();
        self.server_name = source.server_name.clone();
        self.server_port = source.server_port;
    }
}

impl Default for RelaySettings {
    fn default() -> Self {
        RelaySettings {
            server_id: "".to_string(),
            server_name: "".to_string(),
            server_port: 0,
        }
    }
}