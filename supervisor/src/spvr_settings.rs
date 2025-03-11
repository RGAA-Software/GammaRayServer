use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RelayServerConfig {
    pub ip: String,
    pub port: u16,
}

impl Default for RelayServerConfig {
    fn default() -> Self {
        RelayServerConfig {
            ip: "".to_string(),
            port: 0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProfileServerConfig {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SpvrSettings {
    pub server_id: String,
    pub relay_servers: Vec<RelayServerConfig>,
    pub profile_servers: Vec<ProfileServerConfig>,
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
        self.relay_servers = source.relay_servers.clone();
        self.profile_servers = source.profile_servers.clone();
    }
}

impl Default for SpvrSettings {
    fn default() -> Self {
        SpvrSettings {
            server_id: "".to_string(),
            relay_servers: vec![],
            profile_servers: vec![],
        }
    }
}