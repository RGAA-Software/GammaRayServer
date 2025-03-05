use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use base::make_ok_resp_message;
use crate::pr_context::PrContext;
use crate::pr_device::PrDevice;
use crate::RespMessage;

pub struct PrDeviceHandler {

}

impl PrDeviceHandler {

    pub async fn create_new_device(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>, query: Query<HashMap<String, String>>) -> Json<RespMessage<PrDevice>> {
        let mut hw_info = query.get("hw_info").unwrap_or(&"".to_string()).clone();
        let mut platform = query.get("platform").unwrap_or(&"".to_string()).clone();
        let db = context.lock().await.database.clone();
        let mut device = loop {
            let new_device_info = context.lock().await.id_generator.lock().await.generate_new_id(&hw_info, &platform);

            let match_device = db.lock().await.find_device_by_id_and_seed(&new_device_info.device_id, &new_device_info.seed).await;
            if let Some(match_device) = match_device {
                // todo: generate new random pwd, update random pwd
                println!("Match exists device: {}", new_device_info.device_id);

                break Some(match_device);
            }
            else {
                let exist_device = db.lock().await.find_device_by_id(&new_device_info.device_id).await;
                if let Some(exist_device) = exist_device {
                    // need to regenerate
                    hw_info = "".to_string();
                    continue;
                }
                else {
                    // this is a new one
                    // save to database
                    let mut device = PrDevice {
                        device_id: new_device_info.device_id,
                        seed: new_device_info.seed,
                        created_timestamp: base::get_current_timestamp(),
                        last_update_timestamp: base::get_current_timestamp(),
                        random_pwd: new_device_info.random_pwd,
                        safety_pwd: "".to_string(),
                    };

                    println!("New device generated: {}", device.device_id);

                    device.random_pwd = format!("{:x}", md5::compute(&device.random_pwd));
                    let ok = db.lock().await.insert_device(device.clone()).await;
                    if ok {
                        break Some(device);
                    }
                    else {
                        break None;
                    }
                }
            }
        };

        // resp
        if let Some(device) = device {
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