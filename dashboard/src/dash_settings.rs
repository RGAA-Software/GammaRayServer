use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DashSettings {
    pub server_port: u16,
    pub db_path: String,
}

impl DashSettings {
    pub fn new() -> Self {
        DashSettings::default()
    }

    pub fn load(&mut self) {
        let toml_content = std::fs::read_to_string("dash_settings.toml")
            .expect("can't read dash_settings.toml");
        let settings: DashSettings = toml::from_str(&toml_content).expect("parse toml failed");
        self.copy_from(&settings);
        tracing::info!("{:#?}", self);
    }

    fn copy_from(&mut self, source: &DashSettings) {
        self.server_port = source.server_port;
    }
}

impl Default for DashSettings {
    fn default() -> Self {
        DashSettings {
            server_port: 0,
            db_path: "".to_string(),
        }
    }
}