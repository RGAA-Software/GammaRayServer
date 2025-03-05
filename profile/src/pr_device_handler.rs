use std::sync::Arc;
use axum::extract::State;
use axum::Json;
use base::make_ok_resp_message;
use crate::pr_context::PrContext;
use crate::pr_device::PrDevice;
use crate::RespMessage;

pub struct PrDeviceHandler {

}

impl PrDeviceHandler {

    pub async fn create_new_device(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>) -> Json<RespMessage<PrDevice>> {
        let db = context.lock().await.database.clone();
        let device = PrDevice {
            device_id: context.lock().await.id_generator.lock().await.generate_new_id("12345".to_string(), "".to_string()),
            created_timestamp: 10,
            last_update_timestamp: 20,
        };
        let r = db.lock().await.insert_device(device.clone()).await;
        if r {
            Json(make_ok_resp_message(device))
        } else {
            Json(RespMessage::<PrDevice>::new(100))
        }
        
    }

    pub async fn query_devices(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>) -> Json<RespMessage<Vec<PrDevice>>> {
        let db = context.lock().await.database.clone();
        let devices = db.lock().await.query_devices(1, 10).await;
        Json(make_ok_resp_message(devices))
    }

}