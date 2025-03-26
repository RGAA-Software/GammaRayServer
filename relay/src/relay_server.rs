use std::cmp::PartialEq;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use axum::extract::{ConnectInfo, Query, State};
use axum::routing::{get, post};
use axum::{extract::ws::{Message, WebSocket, WebSocketUpgrade}, response::IntoResponse, routing::any, Json, Router, ServiceExt};
use axum_extra::TypedHeader;
use futures_util::StreamExt;
use prost::Message as ProstMessage;
use tokio::sync::Mutex;
use crate::relay_context::RelayContext;

use tower_http::{
    services::ServeDir,
};
use base::{json_util, RespMessage};
use protocol::relay::{RelayMessage, RelayMessageType};
use crate::relay_conn::RelayConn;
use crate::relay_conn_mgr::RelayConnManager;
use crate::{relay_message, relay_room_handler};

pub struct RelayServer {
    pub host: String,
    pub port: u16,
    pub context: Arc<Mutex<RelayContext>>,
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
            .route("/ping", get(RelayServer::ping))
            .route("/relay", any(RelayServer::ws_handler))
            .route("/query/room", get(relay_room_handler::hr_query_room))
            .route("/query/rooms", get(relay_room_handler::hr_query_rooms))
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

    pub async fn ping(State(ctx): State<Arc<Mutex<RelayContext>>>) -> Json<RespMessage<String>> {
        Json(RespMessage::<String> {
            code: 200,
            message: "ok".to_string(),
            data: "Pong".to_string(),
        })
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
        let room_mgr = context.lock().await.room_mgr.clone();

        let mut recv_task = tokio::spawn(async move {
            // device id
            let device_id = params.get("device_id").unwrap_or(&"".to_string()).clone();
            // socket sender
            let sender = Arc::new(Mutex::new(sender));

            // www host
            let addr = who.clone().to_string();
            let mut t = addr.splitn(2, ':');
            let client_w3c_host = t.next().unwrap_or("").to_string();

            tracing::info!("connected device id: {}, client w3c host: {}", device_id, client_w3c_host);
            
            // make relay conn
            let relay_conn = RelayConn::new(context.clone(), sender, device_id.clone(), client_w3c_host).await;
            
            // add to manager
            conn_mgr.lock().await.add_connection(device_id.clone(), relay_conn.clone()).await;
        
            // wait for messages
            while let Some(Ok(msg)) = receiver.next().await {
                // print message and break if instructed to do so
                if RelayServer::process_message(context.clone(), relay_conn.clone(), msg, who).await.is_break() {
                    break;
                }
            }
            
            // this connection has disconnected
            // remove connection
            conn_mgr.lock().await.remove_connection(device_id.clone()).await;
            // remote room
            room_mgr.lock().await.destroy_room_by_creator(device_id).await;
        });

        tokio::select! {
            rv_a = (&mut recv_task) => {
                match rv_a {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!("receive task error: {e:?}")
                    }
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

        let room_mgr = context.lock().await.room_mgr.clone();
        match msg {
            Message::Text(data) => {
                // append received data size
                relay_conn.lock().await.append_received_data_size(data.len()).await;
                // parse json
                let value: serde_json::error::Result<serde_json::Value> = serde_json::from_str(data.as_str());
                if let Err(e) = value {
                    tracing::error!("parse json error: {e}, json: {}", data.to_string());
                    return ControlFlow::Break(());
                }
            }
            Message::Binary(data) => {
                relay_conn.lock().await.append_received_data_size(data.len()).await;
                let m = RelayMessage::decode(data.clone());
                if let Err(e) = m {
                    return ControlFlow::Break(());
                }
                let m = m.unwrap();
                let m_type = m.r#type;
                if m_type == RelayMessageType::KRelayHello {
                    relay_conn.lock().await.on_hello(m).await;
                }
                else if m_type == RelayMessageType::KRelayHeartBeat {
                    relay_conn.lock().await.on_heartbeat(m).await;
                }
                else if m_type == RelayMessageType::KRelayError {
                    relay_conn.lock().await.on_error(m).await
                }
                else if m_type == RelayMessageType::KRelayTargetMessage {
                    //relay_conn.lock().await.on_relay(m, data).await;
                    //relay_conn.self.append_received_data_size(om.len()).await;
                    room_mgr.lock().await.on_relay(m, data).await;
                }
                else if m_type == RelayMessageType::KRelayCreateRoom {
                    //relay_conn.lock().await.on_create_room(m, data).await;
                    room_mgr.lock().await.on_create_room(m, data).await;
                }
                else if m_type == RelayMessageType::KRelayRequestControl {
                    //relay_conn.lock().await.on_request_control(m, data).await;
                    room_mgr.lock().await.on_request_control(m, data).await;
                }
                else if m_type == RelayMessageType::KRelayRequestControlResp {
                    room_mgr.lock().await.on_request_control_resp(m, data).await;
                }
                return ControlFlow::Continue(());
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