mod dash_settings;
mod dash_context;
mod dash_server;
mod dash_conn;
mod dash_conn_mgr;
mod dash_database;
mod dash_group;
mod dash_element;

use std::sync::Arc;
use tokio::sync::Mutex;
use base::log_util;
use crate::dash_conn_mgr::DashConnManager;
use crate::dash_context::DashContext;
use crate::dash_database::DashDatabase;
use crate::dash_server::DashServer;
use crate::dash_settings::DashSettings;

lazy_static::lazy_static! {
    pub static ref gDashSettings: Arc<Mutex<DashSettings>> = Arc::new(Mutex::new(DashSettings::new()));
    pub static ref gDashContext: Arc<Mutex<DashContext>> = Arc::new(Mutex::new(DashContext::new()));
    pub static ref gDashConnManager: Arc<Mutex<DashConnManager>> = Arc::new(Mutex::new(DashConnManager::new()));
    pub static ref gDashDatabase: Arc<Mutex<DashDatabase>> = Arc::new(Mutex::new(DashDatabase::new()));
}

#[tokio::main]
async fn main() {
    // log
    let _guard = log_util::init_log("logs/dashboard/".to_string(), "log_dashboard".to_string());

    // settings
    gDashSettings.lock().await.load();

    // context
    let context = Arc::new(Mutex::new(DashContext::new()));

    // server
    let server = DashServer::new(gDashSettings.lock().await.server_port, context);
    server.start().await;
}
