use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SpvrSettings {
    #[serde(skip_deserializing, skip_serializing)]
    pub server_id: String,
    pub server_name: String,
    pub server_port: u16,
}

impl SpvrSettings {
    pub fn new() -> Self {
        SpvrSettings::default()
    }

    pub fn load(&mut self) {
        let toml_content = std::fs::read_to_string("spvr_settings.toml")
            .expect("can't read spvr_settings.toml");
        let settings: SpvrSettings = toml::from_str(&toml_content).expect("parse toml failed");
        self.copy_from(&settings);
        println!("{:#?}", self);
    }

    fn copy_from(&mut self, source: &SpvrSettings) {
        self.server_id = source.server_id.clone();
        self.server_name = source.server_name.clone();
        self.server_port = source.server_port;
    }
}

impl Default for SpvrSettings {
    fn default() -> Self {
        SpvrSettings {
            server_id: "".to_string(),
            server_name: "".to_string(),
            server_port: 0,
        }
    }
}