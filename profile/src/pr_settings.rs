use serde::Deserialize;
use sysinfo::{
    Components, Disks, Networks, System,
};
use base::system_info;
use crate::gPrSettings;

#[derive(Debug, Deserialize)]
pub struct PrSettings {
    #[serde(skip_deserializing, skip_serializing)]
    pub server_id: String,
    pub server_name: String,
    pub server_w3c_ip: String,
    pub server_local_ip: String,
    pub server_grpc_port: u16,
    pub server_working_port: u16,
    pub spvr_server_ip: String,
    pub spvr_server_port: u16,
    pub mongodb_url: String,
}

impl PrSettings {
    pub fn new() -> PrSettings {
        PrSettings::default()
    }

    pub async fn load_settings() {
        let toml_content = std::fs::read_to_string("pr_settings.toml")
            .expect("can't read pr_settings.toml");
        let mut ns: PrSettings = toml::from_str(&toml_content).expect("parse toml failed");
        let system_info = system_info::SystemInfo::new();
        ns.server_id = format!("{}-{}", ns.server_name, system_info.server_id);
        
        let mut settings = gPrSettings.lock().await;
        tracing::info!("Settings:\n{:#?}", ns);
        *settings = ns;
    }
}

impl Default for PrSettings {
    fn default() -> Self {
        PrSettings {
            server_id: "".to_string(),
            server_name: "".to_string(),
            server_w3c_ip: "".to_string(),
            server_local_ip: "".to_string(),
            server_grpc_port: 0,
            server_working_port: 0,
            spvr_server_ip: "".to_string(),
            spvr_server_port: 0,
            mongodb_url: "".to_string(),
        }
    }
}