use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct RespMessage<T> where T: Serialize, T: Default {
    pub code: i32,
    pub message: String,
    pub data: T
}

impl <T> RespMessage<T> where T: Serialize, T: Default {
    pub fn new_message(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: T::default(),
        }
    }
    
    pub fn new(code: i32) -> Self {
        Self::new_message(code, "".to_string())
    }
    
}

pub fn make_ok_resp_message<T>(value: T) -> RespMessage<T> where T: Serialize, T: Default {
    RespMessage::<T> {
        code: 200,
        message: "ok".to_string(),
        data: value,
    }
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
