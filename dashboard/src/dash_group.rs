use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct DashGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_stream: u32,
    pub allowed_types: Vec<String>,
}

impl DashGroup {
    pub fn new() -> Self {
        DashGroup::default()
    }
}

impl Default for DashGroup {
    fn default() -> Self {
        DashGroup {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            max_stream: 0,
            allowed_types: vec![],
        }
    }
}