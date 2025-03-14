use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum_extra::TypedHeader;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use base::RespMessage;
use protocol::relay::{RelayMessage, RelayMessageType};
use protocol::spvr_inner::SpvrServerType;
use crate::{gSpvrConnMgr, gSpvrGrpcProfileClientMgr, gSpvrGrpcRelayClientMgr};
use crate::spvr_conn::SpvrConn;
use crate::spvr_context::SpvrContext;
use crate::spvr_grpc_client_mgr_trait::SpvrGrpcClientManager;
use crate::spvr_handler_device::hd_get_device_info;
use crate::spvr_handler_server::hs_get_online_servers;

pub struct SpvrServer {
    pub host: String,
    pub port: u16,
    pub context: Arc<Mutex<SpvrContext>>,
}

impl SpvrServer {
    pub fn new(host: String, port: u16, context: Arc<Mutex<SpvrContext>>) -> Self {
        SpvrServer {
            host,
            port,
            context,
        }
    }

    pub async fn start(&self) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

        let app = Router::new()
            .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
            .route("/", get(SpvrServer::root))
            .route("/inner", any(SpvrServer::ws_handler))
            // server info
            .route("/get/online/servers", get(hs_get_online_servers))
            // device info
            .route("/get/device/info", get(hd_get_device_info))
            .with_state(self.context.clone());
        
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        axum::serve(listener,  app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    }
    
    pub async fn root() -> Json<RespMessage<String>> {
        Json(base::ok_resp_str("Working".to_string()))
    }

    async fn ws_handler(
        State(context): State<Arc<Mutex<SpvrContext>>>,
        query: Query<HashMap<String, String>>,
        ws: WebSocketUpgrade,
        user_agent: Option<TypedHeader<headers::UserAgent>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ) -> impl IntoResponse {
        let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
            user_agent.to_string()
        } else {
            String::from("Unknown browser")
        };
        tracing::info!("ws handshake from {}, agent: {}", addr, user_agent);
        for (k, v) in query.iter() {
            tracing::info!("ws query param {}:{}", k, v);
        }
        let params = query.0.clone();
        ws.on_upgrade(move |socket| {
            SpvrServer::handle_socket(context.clone(), params, socket, addr)
        })
    }

    async fn handle_socket(context: Arc<Mutex<SpvrContext>>,
                           params: HashMap<String, String>,
                           socket: WebSocket,
                           who: SocketAddr) {
        let (sender, mut receiver) = socket.split();

        let mut recv_task = tokio::spawn(async move {
            let server_id = params.get("server_id").unwrap_or(&"".to_string()).clone();
            if server_id.is_empty() {
                tracing::error!("spvr, server_id is empty!");
                return;
            }

            let server_type = params.get("server_type").unwrap_or(&"".to_string()).clone();
            let server_type = server_type.parse::<i32>().unwrap_or(-1);
            if server_type == -1 {
                tracing::error!("spvr, server_type is invalid!");
                return;
            }

            let sender = Arc::new(Mutex::new(sender));
            let spvr_conn = SpvrConn::new(context.clone(), 
                                          sender, 
                                          server_id.clone(),
                                          SpvrServerType::try_from(server_type).unwrap()).await;
            let spvr_conn = Arc::new(Mutex::new(spvr_conn));
            gSpvrConnMgr.lock().await.add_conn(server_id.clone(), spvr_conn.clone()).await;

            while let Some(Ok(msg)) = receiver.next().await {
                // print message and break if instructed to do so
                if SpvrServer::process_message(context.clone(), spvr_conn.clone(), msg, who).await.is_break() {
                    break;
                }
            }

            // remove
            gSpvrConnMgr.lock().await.remove_conn(server_id.clone()).await;
            
            // 
            if server_type == SpvrServerType::KSpvrRelayServer {
                gSpvrGrpcRelayClientMgr.lock().await.on_close(server_id).await;
            }
            else if server_type == SpvrServerType::KSpvrProfileServer {
                gSpvrGrpcProfileClientMgr.lock().await.on_close(server_id).await;
            }
            
        });

        tokio::select! {
            spvr_rv = (&mut recv_task) => {
                match spvr_rv {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("receive task error: {e:?}")
                    }
                }
                recv_task.abort();
            },
        }
    }

    async fn process_message(context: Arc<Mutex<SpvrContext>>,
                             spvr_conn: Arc<Mutex<SpvrConn>>,
                             msg: Message,
                             who: SocketAddr)
                             -> ControlFlow<(), ()> {
        match msg {
            Message::Text(data) => {
                return if spvr_conn.lock().await.process_text_message(data).await {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            }
            Message::Binary(data) => {
                return if spvr_conn.lock().await.process_binary_message(data).await {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            }
            Message::Close(c) => {
                if let Some(cf) = c {
                    println!(">>> {} sent close with code {} and reason `{}`", who, cf.code, cf.reason);
                } else {
                    println!(">>> {who} somehow sent close message without CloseFrame");
                }
                return ControlFlow::Break(());
            }

            Message::Pong(v) => {}
            Message::Ping(v) => {}
        }
        ControlFlow::Continue(())
    }

}