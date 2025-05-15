use serde::Deserialize;
use crate::gSpvrSettings;

#[derive(Debug, Deserialize, Clone)]
pub struct SpvrSettings {
    #[serde(skip_deserializing, skip_serializing)]
    pub server_id: String,
    pub server_name: String,
    pub server_port: u16,
    pub single_deploy: bool,
    pub show_ui: bool,
}

impl SpvrSettings {
    pub fn new() -> Self {
        SpvrSettings::default()
    }
    
    pub async fn load_settings() {
        let toml_content = std::fs::read_to_string("spvr_settings.toml")
            .expect("can't read spvr_settings.toml");
        let ns: SpvrSettings = toml::from_str(&toml_content).expect("parse toml failed");
        println!("{:#?}", ns);
        let mut settings = gSpvrSettings.lock().await; // 获取异步锁
        *settings = ns; // 修改内部数据
    }
}

impl Default for SpvrSettings {
    fn default() -> Self {
        SpvrSettings {
            server_id: "".to_string(),
            server_name: "".to_string(),
            server_port: 0,
            single_deploy: false,
            show_ui: false,
        }
    }
}