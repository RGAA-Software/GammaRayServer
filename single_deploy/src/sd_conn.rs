use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures_util::stream::SplitSink;
use tokio::sync::Mutex;
use protocol::sd_inner::{SdInnerMessage, SdInnerMessageType};
use crate::sd_context::SdContext;
use prost::Message as ProstMessage;

pub type SdConnPtr = Arc<Mutex<SdConn>>;

pub struct SdConn {
    pub context: Arc<Mutex<SdContext>>,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub server_id: String,
    pub server_name: String,
}

impl SdConn {
    pub async fn new(context: Arc<Mutex<SdContext>>,
                     sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
                     server_id: String,

    ) -> SdConn {
        SdConn {
            context,
            sender,
            server_id,
            server_name: "".to_string(),
        }
    }

    pub async fn process_text_message(&self, data: Utf8Bytes) -> bool {
        let value: serde_json::error::Result<serde_json::Value> = serde_json::from_str(data.as_str());
        if let Err(e) = value {
            tracing::error!("sd_conn parse json error: {e}, json: {}", data.to_string());
            return false;
        }

        true
    }

    pub async fn process_binary_message(&mut self, data: Bytes) -> bool {
        let m = SdInnerMessage::decode(data);
        if let Err(e) = m {
            tracing::error!("parse error: {:?}", e);
            return false;
        }
        let m = m.unwrap();
        let server_id = m.server_id;
        let server_type = m.server_type;
        let msg_type = m.msg_type;
        if msg_type == SdInnerMessageType::KSdInnerHello {
            // let m_hello = m.hello.unwrap();
            // self.server_name = m_hello.server_name.clone();
            // self.server_w3c_ip = m_hello.server_w3c_ip.to_string();
            // self.server_local_ip = m_hello.server_local_ip.to_string();
            // self.server_grpc_port = m_hello.server_grpc_port as u16;
            // self.server_working_port = m_hello.server_working_port as u16;
            // if server_type == SpvrServerType::KSpvrRelayServer {
            //     gSpvrGrpcRelayClientMgr.lock().await.on_hello(server_id.clone(), m_hello).await
            // }
            // else if server_type == SpvrServerType::KSpvrProfileServer{
            //     gSpvrGrpcProfileClientMgr.lock().await.on_hello(server_id.clone(), m_hello).await;
            // }
        }
        else if msg_type == SdInnerMessageType::KSdInnerHeartBeat {
            let m_heartbeat = m.heartbeat.unwrap();
            // self.server_hb_index = m_heartbeat.hb_index;
            // tracing::info!("heartbeat from: {}, index: {}", server_id, self.server_hb_index);
            // if server_type == SpvrServerType::KSpvrRelayServer {
            //     gSpvrGrpcRelayClientMgr.lock().await.on_heartbeat(server_id.clone(), m_heartbeat).await;
            // }
            // else if server_type == SpvrServerType::KSpvrProfileServer{
            //     gSpvrGrpcProfileClientMgr.lock().await.on_heartbeat(server_id.clone(), m_heartbeat).await;
            // }
        }

        true
    }

}