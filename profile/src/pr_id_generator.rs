pub struct PrIdGenerator {
    
}

impl PrIdGenerator {
    pub fn new() -> std::sync::Arc<tokio::sync::Mutex<PrIdGenerator>> {
        std::sync::Arc::new(tokio::sync::Mutex::new(PrIdGenerator{}))
    }
    
    pub async fn init(&self) {
        
    }
    
    pub fn generate_new_id(&self, info: String, platform: String) -> String {
        let ignore_info = false;
        let target_id = loop {
            let mut seed = info.clone();
            if info.is_empty() || ignore_info {
                seed = uuid::Uuid::new_v4().to_string();
            }

            let mut target_id = "".to_string();
            let digest = md5::compute(seed);
            let digest = format!("{:x}", digest);
            for (index, value) in digest.as_bytes().iter().enumerate() {
                if index == 0 || index == 7 || index == 11
                    || index == 16 || index == 18 || index == 23
                    || index == 26 || index == 28 || index == 30 {
                    let v = (*value) % 10;
                    target_id += &v.to_string();
                    println!("index: {}-> val: {}", index, v);
                }
            }

            println!("target_id: {}", target_id);

            break target_id;
        };

        target_id
    }
    
}