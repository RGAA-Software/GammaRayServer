use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::{Json, Router};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum_extra::TypedHeader;
use futures_util::StreamExt;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use base::RespMessage;
use crate::gSdConnMgr;
use crate::sd_conn::SdConn;
use crate::sd_context::SdContext;

pub struct SdServer {
    pub host: String,
    pub port: u16,
    pub context: Arc<Mutex<SdContext>>,
}

impl SdServer {
    pub fn new(host: String, port: u16, context: Arc<Mutex<SdContext>>) -> Self {
        SdServer {
            host,
            port,
            context,
        }
    }

    pub async fn start(&self) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

        let app = Router::new()
            .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
            .route("/ping", get(SdServer::ping))
            .route("/inner", any(SdServer::ws_handler))
            .with_state(self.context.clone());

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        axum::serve(listener,  app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    }

    pub async fn ping(State(ctx): State<Arc<Mutex<SdContext>>>) -> Json<RespMessage<String>> {
        Json(RespMessage::<String> {
            code: 200,
            message: "ok".to_string(),
            data: "pong".to_string(),
        })
    }

    async fn ws_handler(
        State(context): State<Arc<Mutex<SdContext>>>,
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
            SdServer::handle_socket(context.clone(), params, socket, addr)
        })
    }

    async fn handle_socket(context: Arc<Mutex<SdContext>>,
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
            let sd_conn = SdConn::new(context.clone(),
                                          sender,
                                          server_id.clone()).await;
            let sd_conn = Arc::new(Mutex::new(sd_conn));
            gSdConnMgr.lock().await.add_conn(server_id.clone(), sd_conn.clone()).await;

            while let Some(Ok(msg)) = receiver.next().await {
                // print message and break if instructed to do so
                if SdServer::process_message(context.clone(), sd_conn.clone(), msg, who).await.is_break() {
                    break;
                }
            }

            // remove
            gSdConnMgr.lock().await.remove_conn(server_id.clone()).await;
            

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

    async fn process_message(context: Arc<Mutex<SdContext>>,
                             sd_conn: Arc<Mutex<SdConn>>,
                             msg: Message,
                             who: SocketAddr)
                             -> ControlFlow<(), ()> {
        match msg {
            Message::Text(data) => {
                return if sd_conn.lock().await.process_text_message(data).await {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            }
            Message::Binary(data) => {
                return if sd_conn.lock().await.process_binary_message(data).await {
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