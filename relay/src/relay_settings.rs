use serde::Deserialize;
use sysinfo::{
    Components, Disks, Networks, System,
};
use base::system_info;

#[derive(Debug, Deserialize)]
pub struct RelaySettings {
    #[serde(skip_deserializing, skip_serializing)]
    pub server_id: String,
    pub server_name: String,
    pub server_w3c_ip: String,
    pub server_local_ip: String,
    pub server_grpc_port: u16,
    pub server_working_port: u16,
    pub spvr_server_ip: String,
    pub spvr_server_port: u16,
    pub redis_url: String,
    pub single_deploy_port: u16,
}

impl RelaySettings {
    pub fn new() -> RelaySettings {
        RelaySettings::default()
    }

    pub async fn load(&mut self) {
        let toml_content = std::fs::read_to_string("relay_settings.toml")
            .expect("can't read relay_settings.toml");
        let settings: RelaySettings = toml::from_str(&toml_content).expect("parse toml failed");
        self.copy_from(&settings);

        let system_info = system_info::SystemInfo::new();
        self.server_id = format!("{}-{}", settings.server_name, system_info.server_id);
        
        println!("{:#?}", self);
    }

    fn copy_from(&mut self, source: &RelaySettings) {
        self.server_id = source.server_id.clone();
        self.server_name = source.server_name.clone();
        self.server_w3c_ip = source.server_w3c_ip.clone();
        self.server_local_ip = source.server_local_ip.clone();
        self.server_grpc_port = source.server_grpc_port;
        self.server_working_port = source.server_working_port;
        self.spvr_server_ip = source.spvr_server_ip.clone();
        self.spvr_server_port = source.spvr_server_port;
        self.redis_url = source.redis_url.clone();
    }
}

impl Default for RelaySettings {
    fn default() -> Self {
        RelaySettings {
            server_id: "".to_string(),
            server_name: "".to_string(),
            server_w3c_ip: "".to_string(),
            server_local_ip: "".to_string(),
            server_grpc_port: 0,
            server_working_port: 0,
            spvr_server_ip: "".to_string(),
            spvr_server_port: 0,
            redis_url: "".to_string(),
            single_deploy_port: 30400,
        }
    }
}