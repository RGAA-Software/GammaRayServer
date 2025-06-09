use crate::gRelaySettings;
use serde::Deserialize;
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

    #[serde(skip_deserializing, skip_serializing)]
    pub single_deploy_port: u16,
}

impl RelaySettings {
    pub fn new() -> RelaySettings {
        RelaySettings::default()
    }

    pub async fn load_settings() {
        let toml_content = std::fs::read_to_string("relay_settings.toml")
            .expect("can't read relay_settings.toml");
        let mut ns: RelaySettings = toml::from_str(&toml_content).expect("parse toml failed");
        let system_info = system_info::SystemInfo::new();
        ns.server_id = format!("{}-{}", ns.server_name, system_info.server_id);
        tracing::info!("{:#?}", ns);
        
        let mut settings = gRelaySettings.lock().await;
        *settings = ns;
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