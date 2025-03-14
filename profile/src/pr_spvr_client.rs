use std::sync::Arc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use std::time::Duration;
use axum::body::Bytes;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello, SpvrInnerMessage, SpvrInnerMessageType, SpvrServerType};

// [this]Pr client ---> Supervisor ws server
pub struct PrSpvrClient {
    sender: Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteMessage>>>>
}

impl PrSpvrClient {
    pub fn new() -> PrSpvrClient {
        PrSpvrClient {
            sender: Arc::new(Default::default()),
        }
    }

    pub async fn connect(&self, address: String) {
        let self_sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                let ws_stream = match connect_async(address.clone()).await {
                    Ok((mut stream, response)) => {
                        tracing::info!("Connected to {}", address);
                        let mut m = SpvrInnerMessage::default();
                        m.server_id = "pr_01".to_string();
                        m.server_type = SpvrServerType::KSpvrProfileServer as i32;
                        m.msg_type = i32::from(SpvrInnerMessageType::KSpvrInnerHello);
                        m.hello = Some(SpvrInnerHello {
                            server_name: "xxxx".to_string(),
                            server_w3c_ip: "127.0.0.1".to_string(),
                            server_local_ip: "127.0.0.1".to_string(),
                            server_grpc_port: 40600,
                            server_working_port: 30600,
                        });
                        let _ = stream.send(TungsteniteMessage::Binary(Bytes::from(m.encode_to_vec()))).await;

                        Some(stream)
                    }
                    Err(e) => {
                        tracing::error!("Failed to connect to {}: {}", address, e);
                        None
                    }
                };

                if let Some(stream) = ws_stream {
                    let (sender, mut receiver) = stream.split();
                    *self_sender.lock().await = Some(sender);

                    // heartbeat loop
                    let sender = self_sender.clone();
                    tokio::spawn(async move {
                        let mut hb_index = 0;
                        loop {
                            if let Some(sender) = &mut *sender.lock().await {
                                let mut m = SpvrInnerMessage::default();
                                m.server_id = "pr_01".to_string();
                                m.server_type = SpvrServerType::KSpvrProfileServer as i32;
                                m.msg_type = i32::from(SpvrInnerMessageType::KSpvrInnerHeartBeat);
                                m.heartbeat = Some(SpvrInnerHeartBeat {
                                    hb_index,
                                });
                                let r = sender.send(TungsteniteMessage::Binary(Bytes::from(m.encode_to_vec()))).await;
                                if r.is_err() {
                                    tracing::error!("Sending heartbeat failed, Break the heartbeat loop: {}", hb_index);
                                    break;
                                }
                                hb_index += 1;
                            }
                            else {
                                tracing::error!("Break the heartbeat: {}", hb_index);
                                break;
                            }
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    });

                    // receive message
                    while let Some(msg) = receiver.next().await {
                        match msg {
                            Ok(TungsteniteMessage::Binary(data)) => {
                                println!("Received data: {}", data.len());
                            }
                            Ok(TungsteniteMessage::Text(text)) => {
                                println!("Received message: {}", text);
                            }
                            Ok(TungsteniteMessage::Close(_)) => {
                                println!("Connection closed by server");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error receiving message: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }

                tracing::warn!("will reconnect to {}", address);
                sleep(Duration::from_secs(5)).await;
            }
        });



    }
}