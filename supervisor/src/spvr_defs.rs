pub const KEY_SERVER_ID: &'static str = "server_id";
pub const KEY_SERVER_NAME: &'static str = "server_name";
pub const KEY_SERVER_TYPE: &'static str = "server_type";
pub const KEY_W3C_IP: &'static str = "w3c_ip";
pub const KEY_LOCAL_IP: &'static str = "local_ip";
pub const KEY_GRPC_PORT: &'static str = "grpc_port";
pub const KEY_WORKING_PORT: &'static str = "working_port";
pub const KEY_DEVICE_ID: &'static str = "device_id";
pub const KEY_DEVICE_W3C_IP: &'static str = "device_w3c_ip";
pub const KEY_DEVICE_LOCAL_IPS: &'static str = "device_local_ips";
pub const KEY_RELAY_SERVER_IP: &'static str = "relay_server_ip";
pub const KEY_RELAY_SERVER_PORT: &'static str = "relay_server_port";

// !!! deprecated !!!
// reps a device
pub struct SpvrGrpcDeviceInfo {
    pub device_id: String,
    pub client_w3c_ip: String,
    pub client_local_ips: Vec<String>,
    pub server_w3c_ip: String,
    pub server_working_port: i32,
}