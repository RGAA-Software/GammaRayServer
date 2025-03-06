use base::RespMsgPair;

pub const ERR_PARAM_INVALID: i32 = 600;
pub const ERR_OPERATE_DB_FAILED: i32 = 601;
pub const ERR_DEVICE_NOT_FOUND: i32 = 602;

pub fn get_err_msg_pair(code: i32) -> RespMsgPair {
    if code == ERR_PARAM_INVALID {
        return RespMsgPair {
            code,
            message: "Invalid params".to_string(),
        };
    }
    else if code == ERR_OPERATE_DB_FAILED {
        return RespMsgPair {
            code,
            message: "Operate DB failed".to_string(),
        }
    }
    else if code == ERR_DEVICE_NOT_FOUND {
        return RespMsgPair {
            code,
            message: "Device not found".to_string(),
        }
    }
    
    RespMsgPair {
        code,
        message: format!("Unknown error code {}", code)
    }
}