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
use tokio::sync::Mutex;
use crate::relay_context::RelayContext;

use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

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
        State(context): State<Arc<tokio::sync::Mutex<RelayContext>>>,
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

    async fn handle_socket(context: Arc<Mutex<RelayContext>>, params: HashMap<String, String>, mut socket: WebSocket, who: SocketAddr) {
        let (mut sender, mut receiver) = socket.split();

        let conn_mgr = context.lock().await.device_conn_mgr.clone();
        let mut recv_task = tokio::spawn(async move {
            // todo:
            // save to redis
            let device_id = params.get("device_id").unwrap_or(&"".to_string()).clone();
            conn_mgr.lock().await.add_connection(&device_id).await;

            let mut cnt = 0;
            while let Some(Ok(msg)) = receiver.next().await {
                cnt += 1;
                // print message and break if instructed to do so
                if RelayServer::process_message(context.clone(), msg, who).await.is_break() {
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

    async fn process_message(context: Arc<Mutex<RelayContext>>, msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
        match msg {
            Message::Text(t) => {
                println!(">>> {who} sent str: {t:?}");
            }
            Message::Binary(d) => {
                println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
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
                println!(">>> {who} sent pong with {v:?}");
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