use std::cmp::PartialEq;
use prost::Message;
use protocol::relay::{RelayErrorCode, RelayErrorMessage, RelayMessage, RelayMessageType};

pub fn make_error_message(code: RelayErrorCode) -> Vec<u8> {
    let mut rl_msg = RelayMessage::default();
    rl_msg.set_type(RelayMessageType::KRelayError);
    rl_msg.relay_error = Some(RelayErrorMessage {
        code: code as i32,
        message: get_error_message(code),
        which_message: 0,
    });
    rl_msg.encode_to_vec()
}

pub fn get_error_message(code: RelayErrorCode) -> String {
    if code == RelayErrorCode::KRelayCodeOk {
        return "Ok".to_string();
    }
    else if code == RelayErrorCode::KRelayCodeCreateRoomFailed {
        return "Create room failed.".to_string();
    }

    "".to_string()
}