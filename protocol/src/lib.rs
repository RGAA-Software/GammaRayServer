use crate::relay::RelayMessageType;
use crate::spvr_inner::{SpvrInnerMessageType, SpvrServerType};

pub mod relay;
pub mod grpc_profile;
pub mod grpc_relay;
pub mod grpc_base;
pub mod spvr_inner;

impl PartialEq<RelayMessageType> for i32 {
    fn eq(&self, other: &RelayMessageType) -> bool {
        *self == (*other as i32)
    }
}

impl PartialEq<SpvrInnerMessageType> for i32 {
    fn eq(&self, other: &SpvrInnerMessageType) -> bool {
        *self == (*other as i32)
    }
}

impl PartialEq<SpvrServerType> for i32 {
    fn eq(&self, other: &SpvrServerType) -> bool {
        *self == (*other as i32)
    }
}