use std::cmp::PartialEq;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::routing::{get, post};
use axum::{body::Bytes, extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade}, response::IntoResponse, routing::any, Router, ServiceExt};
use axum_extra::TypedHeader;
use futures_util::StreamExt;
use prost::Message as ProstMessage;
use tokio::sync::Mutex;
use crate::relay_context::RelayContext;

use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tower_http::follow_redirect::policy::PolicyExt;
use crate::proto::tc::{RelayMessage, RelayMessageType};
use crate::relay_conn::RelayConn;
use crate::relay_conn_mgr::RelayConnManager;

pub struct RelayServer {
    pub host: String,
    pub port: u16,
    pub context: Arc<Mutex<RelayContext>>,
}

impl PartialEq<RelayMessageType> for i32 {
    fn eq(&self, other: &RelayMessageType) -> bool {
        *self == (*other as i32)
    }
}

impl RelayServer {
    pub fn new(host: String, port: u16, context: Arc<Mutex<RelayContext>>) -> RelayServer {
        RelayServer {
            host,
            port,
            context,
        }
    }

    pub async fn start(&self) {
        let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

        let app = Router::new()
            .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
            .route("/", get(RelayServer::root))
            .route("/relay", any(RelayServer::ws_handler))
            .with_state(self.context.clone());
            // .layer(
            //     TraceLayer::new_for_http()
            //         .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            // );

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        //axum::serve(listener, app).await.unwrap();
        axum::serve(listener,  app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
    }

    pub async fn root(State(ctx): State<Arc<Mutex<RelayContext>>>) -> &'static str {
        "Hello, World!"
    }

    async fn ws_handler(
        State(context): State<Arc<Mutex<RelayContext>>>,
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
            RelayServer::handle_socket(context.clone(), params, socket, addr)
        })
    }

    async fn handle_socket(context: Arc<Mutex<RelayContext>>,
                           params: HashMap<String, String>,
                           socket: WebSocket,
                           who: SocketAddr) {
        let (sender, mut receiver) = socket.split();
        let conn_mgr = context.lock().await.conn_mgr.clone();

        let mut recv_task = tokio::spawn(async move {
            let device_id = params.get("device_id").unwrap_or(&"".to_string()).clone();
            let sender = Arc::new(Mutex::new(sender));
            let relay_conn = RelayConn::new(context.clone(), sender, device_id.clone());

            conn_mgr.lock().await.add_connection(&device_id, relay_conn.clone()).await;

            let mut cnt = 0;
            while let Some(Ok(msg)) = receiver.next().await {
                cnt += 1;
                // print message and break if instructed to do so
                if RelayServer::process_message(context.clone(), relay_conn.clone(), msg, who).await.is_break() {
                    break;
                }
            }

            // remove
            conn_mgr.lock().await.remove_connection(&device_id).await;
            cnt
        });

        tokio::select! {
            rv_a = (&mut recv_task) => {
                match rv_a {
                    Ok(a) => println!("{a} messages sent to {who}"),
                    Err(a) => println!("Error sending messages {a:?}")
                }
                recv_task.abort();
            },
        }
    }

    async fn process_message(context: Arc<Mutex<RelayContext>>,
                             relay_conn: Arc<Mutex<RelayConn>>,
                             msg: Message,
                             who: SocketAddr)
        -> ControlFlow<(), ()> {

        let conn_mgr = context.lock().await.conn_mgr.clone();
        match msg {
            Message::Text(t) => {

            }
            Message::Binary(data) => {
                let data = data;
                let m = RelayMessage::decode(data.clone());
                if let Err(e) = m {
                    return ControlFlow::Break(());
                }
                let m = m.unwrap();
                let m_type = m.r#type;
                if m_type == RelayMessageType::KRelayHello {
                    relay_conn.lock().await.on_hello().await;
                }
                else if m_type == RelayMessageType::KRelayHeartBeat {
                    relay_conn.lock().await.on_heartbeat().await;
                }
                else if m_type == RelayMessageType::KRelayError {
                    relay_conn.lock().await.on_error(m).await;
                }
                else if m_type == RelayMessageType::KRelayTargetMessage {
                    relay_conn.lock().await.on_relay(m, data).await;
                }
                else if m_type == RelayMessageType::KRelayCreateRoom {

                }
                else if m_type == RelayMessageType::KRelayRequestControl {

                }
                else if m_type == RelayMessageType::KRelayRequestControlResp {

                }
            }
            Message::Close(c) => {
                if let Some(cf) = c {
                    println!(
                        ">>> {} sent close with code {} and reason `{}`",
                        who, cf.code, cf.reason
                    );
                } else {
                    println!(">>> {who} somehow sent close message without CloseFrame");
                }
                return ControlFlow::Break(());
            }

            Message::Pong(v) => {

            }
            // You should never need to manually handle Message::Ping, as axum's websocket library
            // will do so for you automagically by replying with Pong and copying the v according to
            // spec. But if you need the contents of the pings you can see them here.
            Message::Ping(v) => {
                println!(">>> {who} sent ping with {v:?}");
            }
        }
        ControlFlow::Continue(())
    }

}