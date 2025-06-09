use crate::spvr_conn::SpvrConnPtr;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct SpvrEvent {

}

pub struct SpvrEventManager {
    pub tx: Option<Arc<Mutex<Sender<SpvrEvent>>>>,
}

impl SpvrEventManager {
    pub fn new() -> SpvrEventManager {
        SpvrEventManager {
            tx: None,
        }
    }

    pub async fn init(&mut self) {
        let (tx, rx) = broadcast::channel::<SpvrEvent>(1024);
        self.tx = Some(Arc::new(Mutex::new(tx)));
    }

    pub async fn broadcast(&mut self, event: SpvrEvent) {
        if let Err(e) = self.tx.clone().unwrap().lock().await.send(event) {
            tracing::error!("failed to broadcast SpvrEvent: {}", e);
        }
    }

    pub async fn obtain_receiver(&self) -> Arc<Mutex<Receiver<SpvrEvent>>> {
        let r = self.tx.clone().unwrap().lock().await.subscribe();
        Arc::new(Mutex::new(r))
    }

    pub async fn subscribe<F, Fut>(&self, callback: Arc<Mutex<F>>)
        where  F: Fn(SpvrEvent) -> Fut,
               Fut: Future<Output = ()>,
    {
        let rx = self.obtain_receiver().await;
        tokio::spawn(async move {
            while let Ok(msg) = rx.lock().await.recv().await {
                println!("Consumer received: {:#?}", msg);
            }
        });
    }
}