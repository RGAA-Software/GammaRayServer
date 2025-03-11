pub mod json_util;
pub mod string_util;
pub mod system_info;
pub mod log_util;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub type StringMap = HashMap<String, String>;
pub type RespStringMap = RespMessage<StringMap>;
pub type RespVecStringMap = RespMessage<Vec<StringMap>>;

#[derive(Serialize, Debug, Deserialize)]
pub struct RespMessage<T> where T: Serialize, T: Default {
    pub code: i32,
    pub message: String,
    pub data: T
}

pub struct RespMsgPair {
    pub code: i32,
    pub message: String,
}

impl <T> RespMessage<T> where T: Serialize, T: Default {
    pub fn new_message(code: i32, message: String, data: T) -> Self {
        Self {
            code,
            message,
            data: T::default(),
        }
    }
    
    pub fn new(code: i32) -> Self {
        RespMessage::<T>::new_message(code, String::new(), T::default())
    }

    pub fn new_pair(pair: RespMsgPair) -> Self {
        RespMessage::<T>::new_message(pair.code, pair.message, T::default())
    }

    pub fn ok() -> Self{
        RespMessage::<T>::new_message(200, "ok".to_string(), T::default())
    }

    pub fn ok_str(msg: String) -> Self {
        RespMessage::<T>::new_message(200, msg, T::default())
    }
}

pub fn ok_resp<T>(value: T) -> RespMessage<T> where T: Serialize, T: Default {
    RespMessage::<T> {
        code: 200,
        message: "ok".to_string(),
        data: value,
    }
}

pub fn resp_empty_str(pair: RespMsgPair) -> RespMessage<String>{
    RespMessage::<String> {
        code: pair.code,
        message: pair.message,
        data: String::new(),
    }
}

pub fn resp_empty_str_map(pair: RespMsgPair) -> RespStringMap {
    RespMessage::<StringMap> {
        code: pair.code,
        message: pair.message,
        data: StringMap::new(),
    }
}

pub fn resp_empty_vec_str_map(pair: RespMsgPair) -> RespVecStringMap {
    RespMessage::<Vec<StringMap>> {
        code: pair.code,
        message: pair.message,
        data: Vec::new(),
    }
}

pub fn ok_resp_str(data: String) -> RespMessage<String> {
    RespMessage::<String> {
        code: 200,
        message: "ok".to_string(),
        data,
    }
}

pub fn ok_resp_str_map(data: HashMap<String, String>) -> RespStringMap {
    RespMessage::<StringMap> {
        code: 200,
        message: "ok".to_string(),
        data,
    }
}

pub fn ok_resp_vec_str_map(data: Vec<HashMap<String, String>>) -> RespVecStringMap {
    RespMessage::<Vec<StringMap>> {
        code: 0,
        message: "".to_string(),
        data,
    }
}

pub fn get_query_param(params: &HashMap<String, String>, key: &str) -> Option<String> {
    let value = params.get(key);
    if let Some(value) = value {
        Some(value.to_string())
    }
    else {
        None
    }
}

pub fn get_current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn md5_hex(input: &String) -> String {
    format!("{:x}", md5::compute(input))
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
