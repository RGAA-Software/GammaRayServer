use std::collections::HashMap;
use std::ops::ControlFlow;
use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::stream::SplitSink;
use prost::Message as ProstMessage;
use tokio::sync::Mutex;
use base::{StrMap, StringMap};
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello, SpvrInnerMessage, SpvrInnerMessageType, SpvrServerType};
use crate::{gSpvrGrpcProfileClientMgr, gSpvrGrpcRelayClientMgr};
use crate::spvr_context::SpvrContext;
use crate::spvr_defs::{KEY_GRPC_PORT, KEY_LOCAL_IP, KEY_SERVER_ID, KEY_SERVER_NAME, KEY_SERVER_TYPE, KEY_W3C_IP, KEY_WORKING_PORT};
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;

pub type SpvrConnPtr = Arc<Mutex<SpvrConn>>;

pub struct SpvrConn {
    pub context: Arc<Mutex<SpvrContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub server_id: String,
    pub server_name: String,
    pub server_type: SpvrServerType,
    pub server_w3c_ip: String,
    pub server_local_ip: String,
    pub server_grpc_port: u16,
    pub server_working_port: u16,
    pub server_hb_index: i64,
}

impl SpvrConn {
    pub async fn new(context: Arc<Mutex<SpvrContext>>,
                     sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
                     server_id: String, server_type: SpvrServerType,

    ) -> SpvrConn {
        SpvrConn {
            context,
            sender,
            server_id,
            server_name: "".to_string(),
            server_type,
            server_w3c_ip: "".to_string(),
            server_local_ip: "".to_string(),
            server_grpc_port: 0,
            server_working_port: 0,
            server_hb_index: 0,
        }
    }
    
    pub async fn process_binary_message(&mut self, data: Bytes) -> bool {
        let m = SpvrInnerMessage::decode(data);
        if let Err(e) = m {
            tracing::error!("parse error: {:?}", e);
            return false;
        }
        let m = m.unwrap();
        let server_id = m.server_id;
        let server_type = m.server_type;
        let msg_type = m.msg_type;
        if msg_type == SpvrInnerMessageType::KSpvrInnerHello {
            let m_hello = m.hello.unwrap();
            self.server_name = m_hello.server_name.clone();
            self.server_w3c_ip = m_hello.server_w3c_ip.to_string();
            self.server_local_ip = m_hello.server_local_ip.to_string();
            self.server_grpc_port = m_hello.server_grpc_port as u16;
            self.server_working_port = m_hello.server_working_port as u16;
            if server_type == SpvrServerType::KSpvrRelayServer {
                gSpvrGrpcRelayClientMgr.lock().await.on_hello(server_id.clone(), m_hello).await
            }
            else if server_type == SpvrServerType::KSpvrProfileServer{
                gSpvrGrpcProfileClientMgr.lock().await.on_hello(server_id.clone(), m_hello).await;
            }
        }
        else if msg_type == SpvrInnerMessageType::KSpvrInnerHeartBeat {
            let m_heartbeat = m.heartbeat.unwrap();
            self.server_hb_index = m_heartbeat.hb_index;
            tracing::info!("heartbeat from: {}, index: {}", server_id, self.server_hb_index);
            if server_type == SpvrServerType::KSpvrRelayServer {
                gSpvrGrpcRelayClientMgr.lock().await.on_heartbeat(server_id.clone(), m_heartbeat).await;
            }
            else if server_type == SpvrServerType::KSpvrProfileServer{
                gSpvrGrpcProfileClientMgr.lock().await.on_heartbeat(server_id.clone(), m_heartbeat).await;
            }
        }

        true
    }

    pub async fn process_text_message(&self, data: Utf8Bytes) -> bool {
        let value: serde_json::error::Result<serde_json::Value> = serde_json::from_str(data.as_str());
        if let Err(e) = value {
            tracing::error!("parse json error: {e}, json: {}", data.to_string());
            return false;
        }
        
        true
    }

    // connected server info; w3cip, ip, grpc port, etc
    // relay server
    // profile server
    pub async fn get_conn_server_info(&self) -> StrMap {
        let mut r = HashMap::new();
        r.insert(KEY_SERVER_ID, self.server_id.clone());
        r.insert(KEY_SERVER_NAME, self.server_name.clone());
        r.insert(KEY_SERVER_TYPE, (self.server_type as i32).to_string());
        r.insert(KEY_W3C_IP, self.server_w3c_ip.clone());
        r.insert(KEY_LOCAL_IP, self.server_local_ip.clone());
        r.insert(KEY_GRPC_PORT, self.server_grpc_port.to_string());
        r.insert(KEY_WORKING_PORT, self.server_working_port.to_string());
        r
    }
}