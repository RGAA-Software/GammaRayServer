use tracing_subscriber::util::SubscriberInitExt;
use crate::pr_database::PrDatabase;
use crate::pr_id_generator::PrIdGenerator;

pub struct PrContext {
    pub id_generator: std::sync::Arc<tokio::sync::Mutex<PrIdGenerator>>,
}

impl PrContext {
    pub fn new() -> Self {
        Self {
            id_generator: PrIdGenerator::new(),
        }
    }

    pub async fn init(&self) -> bool {
        self.id_generator.lock().await.init().await;
        true
    }
}