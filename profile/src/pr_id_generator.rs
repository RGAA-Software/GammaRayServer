use rand::Rng;

pub struct PrIdGenerator {
    
}

pub struct GenDeviceInfo {
    pub device_id: String,
    pub seed: String,
    pub random_pwd: String,
}

impl PrIdGenerator {
    pub fn new() -> std::sync::Arc<tokio::sync::Mutex<PrIdGenerator>> {
        std::sync::Arc::new(tokio::sync::Mutex::new(PrIdGenerator{}))
    }
    
    pub async fn init(&self) {
        
    }
    
    pub fn generate_new_id(&self, info: &String, platform: &String) -> GenDeviceInfo {
        let ignore_info = false;
        let mut seed = info.clone();
        if info.is_empty() || ignore_info {
            seed = uuid::Uuid::new_v4().to_string();
        }

        let mut device_id = "".to_string();
        let digest = base::md5_hex(&seed.clone());
        for (index, value) in digest.as_bytes().iter().enumerate() {
            if index == 0 || index == 7 || index == 11
                || index == 16 || index == 18 || index == 23
                || index == 26 || index == 28 || index == 30 {
                let v = (*value) % 10;
                device_id += &v.to_string();
                println!("index: {}-> val: {}", index, v);
            }
        }

        println!("target_id: {}", device_id);

        GenDeviceInfo {
            device_id,
            seed,
            random_pwd: self.generate_random_pwd(),
        }
    }


    pub fn generate_random_pwd(&self) -> String {
        let mut rng = rand::rng();
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ123456789";
        let password: String = (0..8)
            .map(|_| {
                let idx = rng.random_range(0..charset.len());
                charset[idx] as char
            })
            .collect();
        password
    }
}