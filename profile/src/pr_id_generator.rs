pub struct PrIdGenerator {
    
}

impl PrIdGenerator {
    pub fn new() -> std::sync::Arc<tokio::sync::Mutex<PrIdGenerator>> {
        std::sync::Arc::new(tokio::sync::Mutex::new(PrIdGenerator{}))
    }
    
    pub async fn init(&self) {
        
    }
    
    pub fn generate_new_id() -> String {
        return "".to_string()
    }
    
}