use tracing_subscriber::util::SubscriberInitExt;
use crate::pr_database::PrDatabase;
use crate::pr_id_generator::PrIdGenerator;

pub struct PrContext {
    pub database: std::sync::Arc<tokio::sync::Mutex<PrDatabase>>,
    pub id_generator: std::sync::Arc<tokio::sync::Mutex<PrIdGenerator>>,
}

impl PrContext {
    pub fn new() -> Self {
        Self {
            database: PrDatabase::new(),
            id_generator: PrIdGenerator::new(),
        }
    }

    pub async fn init(&self) -> bool {
        println!("INIT!!!");
        self.database.lock().await.init().await;
        self.id_generator.lock().await.init().await;
        return true;
    }
}