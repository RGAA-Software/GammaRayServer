use crate::relay::RelayMessageType;

pub mod relay;
pub mod grpc_profile;
pub mod grpc_relay;
pub mod grpc_base;

impl PartialEq<RelayMessageType> for i32 {
    fn eq(&self, other: &RelayMessageType) -> bool {
        *self == (*other as i32)
    }
}