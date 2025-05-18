mod spvr_context;
mod spvr_server;
mod spvr_grpc_relay_client;
mod spvr_settings;
mod spvr_conn;
mod spvr_conn_mgr;
mod spvr_grpc_profile_client;
mod spvr_grpc_relay_client_mgr;
mod spvr_grpc_profile_client_mgr;
mod spvr_grpc_client_mgr_trait;
mod spvr_server_handler;
mod spvr_defs;
mod spvr_relay_handler;
mod spvr_event_mgr;
mod spvr_errors;
mod spvr_profile_handler;
mod svpr_ui;

use crate::spvr_conn_mgr::SpvrConnManager;
use crate::spvr_grpc_profile_client_mgr::SpvrGrpcProfileClientMgr;
use crate::spvr_grpc_relay_client_mgr::SpvrGrpcRelayClientManager;
use crate::spvr_server::SpvrServer;
use crate::spvr_settings::SpvrSettings;
use base::log_util;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::spvr_event_mgr::SpvrEventManager;
use eframe::egui;
use tracing::log;

lazy_static::lazy_static! {
    pub static ref gSpvrSettings: Arc<Mutex<SpvrSettings>> = Arc::new(Mutex::new(SpvrSettings::new()));
    pub static ref gSpvrConnMgr: Arc<Mutex<SpvrConnManager>> = Arc::new(Mutex::new(SpvrConnManager::new()));
    pub static ref gSpvrGrpcRelayClientMgr: Arc<Mutex<SpvrGrpcRelayClientManager >> = Arc::new(Mutex::new(SpvrGrpcRelayClientManager::new()));
    pub static ref gSpvrGrpcProfileClientMgr: Arc<Mutex<SpvrGrpcProfileClientMgr >> = Arc::new(Mutex::new(SpvrGrpcProfileClientMgr::new()));
    pub static ref gSpvrEventManager: Arc<Mutex<SpvrEventManager>> = Arc::new(Mutex::new(SpvrEventManager::new()));
}

#[tokio::main]
async fn main() {
    // log
    let _guard = log_util::init_log("logs/supervisor/".to_string(), "log_supervisor".to_string());

    // settings
    SpvrSettings::load_settings().await;
    
    // event manager
    gSpvrEventManager.lock().await.init().await;
    
    // context
    let context = Arc::new(Mutex::new(spvr_context::SpvrContext::new()));
    
    let srv_task = async move {
        // server
        let server = SpvrServer::new("0.0.0.0".to_string(),
                                     gSpvrSettings.lock().await.server_port,
                                     context);
        server.start().await;
    };
    
    if gSpvrSettings.lock().await.show_ui {
        tokio::spawn(async move {
            srv_task.await;
        });

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 540.0]),
            ..Default::default()
        };
        let r = eframe::run_native(
            "GammaRay Server",
            options,
            Box::new(|cc| {
                cc.egui_ctx.set_visuals(egui::Visuals::dark());
                // This gives us image support:
                egui_extras::install_image_loaders(&cc.egui_ctx);
        
                Ok(Box::<svpr_ui::SpvrUI>::default())
            }),
        );
        if let Err(e) = r {
            log::error!("{}", e);
        }
    }
    else {
        srv_task.await;
    }
}