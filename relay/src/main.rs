mod relay_context;
mod relay_server;
mod relay_conn_mgr;
mod relay_conn;
mod proto;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::relay_context::RelayContext;
use crate::relay_server::RelayServer;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let context = RelayContext::new().await;
    if let Err(err) = context {
        tracing::error!("Create RelayContext failed: {}", err);
        return;
    }
    let context = context.unwrap();
    let context = Arc::new(Mutex::new(context));
    context.lock().await.init();

    let server = RelayServer::new("0.0.0.0".to_string(), 20681, context.clone());
    server.start().await;
}
