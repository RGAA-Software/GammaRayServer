use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use tokio::sync::Mutex;
use base::{resp_empty_str, ok_resp, ok_resp_str_map, RespStringMap, resp_empty_str_map};
use crate::pr_context::PrContext;
use crate::pr_device::PrDevice;
use crate::{gDatabase, RespMessage};
use crate::pr_errors::{get_err_pair, ERR_DEVICE_NOT_FOUND, ERR_OPERATE_DB_FAILED, ERR_PARAM_INVALID, ERR_PASSWORD_FAILED};
use crate::pr_message::{KEY_DEVICE_ID, KEY_PWD_TYPE};

pub struct PrDeviceHandler {

}

impl PrDeviceHandler {

    pub async fn create_new_device(State(context): State<Arc<Mutex<PrContext>>>,
                                   query: Query<HashMap<String, String>>)
        -> Json<RespMessage<PrDevice>> {
        let mut hw_info = query.get("hw_info").unwrap_or(&"".to_string()).clone();
        let platform = query.get("platform").unwrap_or(&"".to_string()).clone();
        let device = loop {
            let id_generator = context.lock().await.id_generator.clone();
            let new_device_info = id_generator.lock().await.generate_new_id(&hw_info, &platform);

            tracing::info!("will find in database: {}", new_device_info.device_id.clone());
            let match_device = gDatabase.lock().await
                .find_device_by_id_and_seed(new_device_info.device_id.clone(), new_device_info.seed.clone()).await;
            tracing::info!("find in database? {}", match_device.is_some());

            if let Some(mut match_device) = match_device {
                // todo: generate new random pwd, update random pwd
                tracing::info!("Match exists device: {}", new_device_info.device_id);

                let new_random_pwd = id_generator.lock().await.generate_random_pwd();
                let update_info = HashMap::<String, String>::from([
                    (String::from("random_pwd"), base::md5_hex(&new_random_pwd.clone()))
                ]);
                let update_result = gDatabase.lock().await.update_device(match_device.device_id.clone(), update_info).await;
                if update_result {
                    match_device.random_pwd = new_random_pwd;
                    break Some(match_device);
                }
                else {
                    break None;
                }
            }
            else {
                tracing::info!("to find: {} in database", new_device_info.device_id);
                let exist_device = gDatabase.lock().await.find_device_by_id(new_device_info.device_id.clone()).await;
                if let Some(exist_device) = exist_device {
                    tracing::warn!("already exist: {}, will regenerate!", exist_device.device_id);
                    // need to regenerate
                    hw_info = "".to_string();
                    continue;
                }
                else {
                    tracing::info!("the device is a new one, insert to db: {}", new_device_info.device_id);
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

                    let resp_device = device.clone();

                    device.random_pwd = base::md5_hex(&device.random_pwd);
                    let ok = gDatabase.lock().await.insert_device(device.clone()).await;
                    if ok {
                        break Some(resp_device);
                    }
                    else {
                        tracing::error!("insert failed: {}", device.device_id.clone());
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

    pub async fn query_devices(State(context): State<Arc<Mutex<PrContext>>>,
                               query: Query<HashMap<String, String>>)
        -> Json<RespMessage<Vec<PrDevice>>> {
        let db = gDatabase.clone();
        let devices = db.lock().await.query_devices(1, 10).await;
        Json(ok_resp(devices))
    }

    pub async fn append_used_time(State(context): State<Arc<Mutex<PrContext>>>,
                                  query: Query<HashMap<String, String>>)
        -> Json<RespMessage<String>> {
        let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
        let period = query.get("period").unwrap_or(&"".to_string()).clone();
        let period = period.parse::<i64>().unwrap_or(0);
        if period <= 0 || device_id.is_empty(){
            return Json(resp_empty_str(get_err_pair(ERR_PARAM_INVALID)));
        }

        // exists device
        let device = gDatabase.lock().await.find_device_by_id(device_id.clone()).await;
        if let None = device {
            return Json(resp_empty_str(get_err_pair(ERR_DEVICE_NOT_FOUND)));
        }
        let device = device.unwrap();
        let target_used_time = device.used_time + period;

        let r = gDatabase.lock().await
            .update_device_field(device_id.clone(), "used_time".to_string(), target_used_time).await;
        if r {
            Json(ok_resp(device_id))
        }
        else {
            Json(resp_empty_str(get_err_pair(ERR_OPERATE_DB_FAILED)))
        }
    }

    pub async fn verify_device_info(State(context): State<Arc<Mutex<PrContext>>>,
                                   query: Query<HashMap<String, String>>)
                                   -> Json<RespStringMap> {
        let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
        let random_pwd_md5 = query.get("random_pwd_md5").unwrap_or(&"".to_string()).clone();
        let safety_pwd_md5 = query.get("safety_pwd_md5").unwrap_or(&"".to_string()).clone();
        if device_id.is_empty() || random_pwd_md5.is_empty() {
            return Json(resp_empty_str_map(get_err_pair(ERR_PARAM_INVALID)));
        }

        let device = gDatabase.lock().await.find_device_by_id(device_id.clone()).await;
        if let None = device {
            return Json(resp_empty_str_map(get_err_pair(ERR_DEVICE_NOT_FOUND)));
        }
        let device = device.unwrap();
        let mut ok = device.device_id == device_id;

        let ok_random_pwd = if random_pwd_md5 == device.random_pwd {true} else {false};
        let ok_safety_pwd = if !safety_pwd_md5.is_empty() && safety_pwd_md5 == device.safety_pwd {true} else {false};

        if !ok_random_pwd && !ok_safety_pwd {
            return Json(resp_empty_str_map(get_err_pair(ERR_PASSWORD_FAILED)));
        }

        let pwd_type = if ok_random_pwd {"random"} else if ok_safety_pwd {"safety"} else {"unknown"};

        let mut hm = HashMap::new();
        hm.insert(KEY_DEVICE_ID.to_string(), device_id);
        hm.insert(KEY_PWD_TYPE.to_string(), pwd_type.to_string());
        Json(ok_resp_str_map(hm))
    }

    pub async fn update_random_pwd(State(context): State<Arc<Mutex<PrContext>>>,
                                   query: Query<HashMap<String, String>>)
                                   -> Json<RespMessage<PrDevice>> {
        let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
        //let new_random_pwd = query.get("new_random_pwd").unwrap_or(&"".to_string()).clone();

        let db = gDatabase.clone();
        let device = db.lock().await.find_device_by_id(device_id.clone()).await;
        if let None = device {
            return Json(RespMessage::<PrDevice>::new(ERR_DEVICE_NOT_FOUND));
        }
        let mut device = device.unwrap();

        // generate new random password
        let id_generator = context.lock().await.id_generator.clone();
        let new_random_pwd = id_generator.lock().await.generate_random_pwd();

        // update to database
        let update_info = HashMap::<String, String>::from([
            (String::from("random_pwd"), base::md5_hex(&new_random_pwd.clone()))
        ]);
        let r = db.lock().await.update_device(device_id.clone(), update_info).await;
        if !r {
            return Json(RespMessage::<PrDevice>::new(ERR_DEVICE_NOT_FOUND));
        }

        device.random_pwd = new_random_pwd;
        Json(ok_resp(device))
    }

    pub async fn update_safety_pwd(State(context): State<Arc<Mutex<PrContext>>>,
                                   query: Query<HashMap<String, String>>)
                                   -> Json<RespMessage<PrDevice>> {
        let device_id = query.get("device_id").unwrap_or(&"".to_string()).clone();
        let new_safety_pwd = query.get("new_safety_pwd").unwrap_or(&"".to_string()).clone();
        if device_id.is_empty() || new_safety_pwd.is_empty() {
            return return Json(RespMessage::<PrDevice>::new(ERR_PARAM_INVALID));
        }

        let db = gDatabase.clone();
        let device = db.lock().await.find_device_by_id(device_id.clone()).await;
        if let None = device {
            return Json(RespMessage::<PrDevice>::new(ERR_DEVICE_NOT_FOUND));
        }
        let mut device = device.unwrap();

        // update to database
        let update_info = HashMap::<String, String>::from([
            (String::from("safety_pwd"), base::md5_hex(&new_safety_pwd.clone()))
        ]);
        let r = db.lock().await.update_device(device_id.clone(), update_info).await;
        if !r {
            return Json(RespMessage::<PrDevice>::new(ERR_DEVICE_NOT_FOUND));
        }

        device.safety_pwd = new_safety_pwd;
        Json(ok_resp(device))
    }
}