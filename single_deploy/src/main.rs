mod sd_ui;
mod sd_settings;
mod sd_server;
mod sd_context;
mod sd_conn;
mod sd_conn_mgr;

use std::sync::Arc;
use clap::Parser;
use tokio::sync::Mutex;
use crate::sd_conn_mgr::SdConnManager;
use crate::sd_settings::SdSettings;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Display UI or not
    #[arg(short, long, default_value_t = true)]
    show_ui: bool,
}

lazy_static::lazy_static! {
    pub static ref gSdSettings: Arc<Mutex<SdSettings>> = Arc::new(Mutex::new(SdSettings::new()));
    pub static ref gSdConnMgr: Arc<Mutex<SdConnManager>> = Arc::new(Mutex::new(SdConnManager::new()));
}

#[tokio::main]
async fn main() {

    let args = Args::parse();
    
     if args.show_ui {
         // load from db
         gSdSettings.lock().await.load_from_db().await;
         
         let options = eframe::NativeOptions {
             viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 540.0]),
             ..Default::default()
         };
         let r = eframe::run_native(
             "GammaRay Single Deploy",
             options,
             Box::new(|cc| {
                 cc.egui_ctx.set_visuals(egui::Visuals::dark());
                 // This gives us image support:
                 egui_extras::install_image_loaders(&cc.egui_ctx);

                 Ok(Box::<sd_ui::SpvrUI>::default())
             }),
         );
         if let Err(e) = r {
             log::error!("{}", e);
         }
     }
     else {
         // load from config
         gSdSettings.lock().await.load_from_config().await;
         
     }
    
}