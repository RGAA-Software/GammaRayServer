pub struct DashElement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_stream: u32,
    pub allowed_types: Vec<String>,
}

impl DashElement {
    pub fn new() -> Self {
        DashElement::default()
    }   
}

impl Default for DashElement {
    fn default() -> Self {
        DashElement {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            max_stream: 0,
            allowed_types: vec![],
        }
    }
}