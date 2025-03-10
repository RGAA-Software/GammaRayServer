use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use tokio::task::id;
use base::{resp_empty_str, ok_resp};
use crate::pr_context::PrContext;
use crate::pr_device::PrDevice;
use crate::RespMessage;
use crate::pr_errors::{get_err_pair, ERR_DEVICE_NOT_FOUND, ERR_OPERATE_DB_FAILED, ERR_PARAM_INVALID};

pub struct PrDeviceHandler {

}

impl PrDeviceHandler {

    pub async fn create_new_device(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>, query: Query<HashMap<String, String>>) -> Json<RespMessage<PrDevice>> {
        let mut hw_info = query.get("hw_info").unwrap_or(&"".to_string()).clone();
        let platform = query.get("platform").unwrap_or(&"".to_string()).clone();
        let db = context.lock().await.database.clone();
        let device = loop {
            let id_generator = context.lock().await.id_generator.clone();
            let new_device_info = id_generator.lock().await.generate_new_id(&hw_info, &platform);

            let match_device = db.lock().await.find_device_by_id_and_seed(&new_device_info.device_id, &new_device_info.seed).await;
            if let Some(mut match_device) = match_device {
                // todo: generate new random pwd, update random pwd
                println!("Match exists device: {}", new_device_info.device_id);

                let new_random_pwd = id_generator.lock().await.generate_random_pwd();
                let update_info = HashMap::<String, String>::from([
                    (String::from("random_pwd"), base::md5_hex(&new_random_pwd.clone()))
                ]);
                let update_result = db.lock().await.update_device(&match_device.device_id, update_info).await;
                if update_result {
                    match_device.random_pwd = new_random_pwd;
                    break Some(match_device);
                }
                else {
                    break None;
                }
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
                        user_id: "".to_string(),
                        seed: new_device_info.seed,
                        created_timestamp: base::get_current_timestamp(),
                        last_update_timestamp: base::get_current_timestamp(),
                        random_pwd: new_device_info.random_pwd,
                        safety_pwd: "".to_string(),
                        used_time: 0,
                    };

                    println!("New device generated: {}", device.device_id);

                    let resp_device = device.clone();

                    device.random_pwd = base::md5_hex(&device.random_pwd);
                    let ok = db.lock().await.insert_device(device.clone()).await;
                    if ok {
                        break Some(resp_device);
                    }
                    else {
                        break None;
                    }
                }
            }
        };

        // resp
        if let Some(device) = device {
            Json(ok_resp(device))
        } else {
            Json(RespMessage::<PrDevice>::new(100))
        }
        
    }

    pub async fn query_devices(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>, query: Query<HashMap<String, String>>) -> Json<RespMessage<Vec<PrDevice>>> {
        let db = context.lock().await.database.clone();
        let devices = db.lock().await.query_devices(1, 10).await;
        Json(ok_resp(devices))
    }

    pub async fn append_used_time(State(context): State<Arc<tokio::sync::Mutex<PrContext>>>, query: Query<HashMap<String, String>>) -> Json<RespMessage<String>>  {
        let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
        let period = query.get("period").unwrap_or(&"".to_string()).clone();
        let period = period.parse::<i64>().unwrap_or(0);
        if period <= 0 || device_id.is_empty(){
            return Json(resp_empty_str(get_err_pair(ERR_PARAM_INVALID)));
        }
        let db = context.lock().await.database.clone();
        // exists device
        let device = db.lock().await.find_device_by_id(&device_id).await;
        if let None = device {
            return Json(resp_empty_str(get_err_pair(ERR_DEVICE_NOT_FOUND)));
        }
        let device = device.unwrap();
        let target_used_time = device.used_time + period;

        let r = db.lock().await.update_device_field(&device_id, &"used_time".to_string(), target_used_time).await;
        if r {
            Json(ok_resp(device_id))
        }
        else {
            Json(resp_empty_str(get_err_pair(ERR_OPERATE_DB_FAILED)))
        }
    }

}