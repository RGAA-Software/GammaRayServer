pub struct SpvrSettings {
    pub server_id: String,
}

impl SpvrSettings {
    pub fn new() -> Self {
        SpvrSettings::default()
    }    
}

impl Default for SpvrSettings {
    fn default() -> Self {
        SpvrSettings {
            server_id: "".to_string(),
        }
    }
}