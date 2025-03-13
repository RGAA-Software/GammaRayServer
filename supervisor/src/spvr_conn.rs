use std::ops::ControlFlow;
use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::stream::SplitSink;
use prost::Message as ProstMessage;
use tokio::sync::Mutex;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello, SpvrInnerMessage, SpvrInnerMessageType, SpvrServerType};
use crate::{gSpvrGrpcProfileClientMgr, gSpvrGrpcRelayClientMgr};
use crate::spvr_context::SpvrContext;
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;

pub struct SpvrConn {
    pub context: Arc<Mutex<SpvrContext>>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl SpvrConn {
    pub async fn new(context: Arc<Mutex<SpvrContext>>,
               sender: Arc<Mutex<SplitSink<WebSocket, Message>>>) -> SpvrConn {
        SpvrConn {
            context,
            sender,
        }
    }
    
    pub async fn process_binary_message(&self, data: Bytes) -> bool {
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
            if server_type == SpvrServerType::KSpvrRelayServer {
                gSpvrGrpcRelayClientMgr.lock().await.on_hello(server_id.clone(), m.hello.unwrap()).await
            }
            else if server_type == SpvrServerType::KSpvrProfileServer{
                gSpvrGrpcProfileClientMgr.lock().await.on_hello(server_id.clone(), m.hello.unwrap()).await;
            }
        }
        else if msg_type == SpvrInnerMessageType::KSpvrInnerHeartBeat {
            if server_type == SpvrServerType::KSpvrRelayServer {
                gSpvrGrpcRelayClientMgr.lock().await.on_heartbeat(server_id.clone(), m.heartbeat.unwrap()).await;
            }
            else if server_type == SpvrServerType::KSpvrProfileServer{
                gSpvrGrpcProfileClientMgr.lock().await.on_heartbeat(server_id.clone(), m.heartbeat.unwrap()).await;
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
}