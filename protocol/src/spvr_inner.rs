// This file is @generated by prost-build.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpvrInnerHello {
    #[prost(string, tag = "1")]
    pub server_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub server_w3c_ip: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub server_local_ip: ::prost::alloc::string::String,
    #[prost(uint32, tag = "5")]
    pub server_grpc_port: u32,
    #[prost(uint32, tag = "6")]
    pub server_working_port: u32,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct SpvrInnerHeartBeat {
    #[prost(int64, tag = "1")]
    pub hb_index: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpvrInnerMessage {
    #[prost(enumeration = "SpvrServerType", tag = "1")]
    pub server_type: i32,
    #[prost(enumeration = "SpvrInnerMessageType", tag = "2")]
    pub msg_type: i32,
    #[prost(string, tag = "3")]
    pub server_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "5")]
    pub hello: ::core::option::Option<SpvrInnerHello>,
    #[prost(message, optional, tag = "10")]
    pub heartbeat: ::core::option::Option<SpvrInnerHeartBeat>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpvrServerType {
    KSpvrRelayServer = 0,
    KSpvrProfileServer = 1,
    KSpvrSignalingServer = 2,
}
impl SpvrServerType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::KSpvrRelayServer => "kSpvrRelayServer",
            Self::KSpvrProfileServer => "kSpvrProfileServer",
            Self::KSpvrSignalingServer => "kSpvrSignalingServer",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "kSpvrRelayServer" => Some(Self::KSpvrRelayServer),
            "kSpvrProfileServer" => Some(Self::KSpvrProfileServer),
            "kSpvrSignalingServer" => Some(Self::KSpvrSignalingServer),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpvrInnerMessageType {
    KSpvrInnerHello = 0,
    KSpvrInnerHeartBeat = 1,
}
impl SpvrInnerMessageType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::KSpvrInnerHello => "kSpvrInnerHello",
            Self::KSpvrInnerHeartBeat => "kSpvrInnerHeartBeat",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "kSpvrInnerHello" => Some(Self::KSpvrInnerHello),
            "kSpvrInnerHeartBeat" => Some(Self::KSpvrInnerHeartBeat),
            _ => None,
        }
    }
}
