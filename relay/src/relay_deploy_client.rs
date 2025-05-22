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
use protocol::sd_inner::{SdInnerHeartBeat, SdInnerHello, SdInnerMessage, SdInnerMessageType, SdServerType};
use crate::gRelaySettings;

// [this]Relay client ---> Deploy ws server
pub struct RelayDeployClient {
    sender: Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TungsteniteMessage>>>>
}

impl RelayDeployClient {
    pub fn new() -> RelayDeployClient {
        RelayDeployClient {
            sender: Arc::new(Default::default()),
        }
    }

    pub async fn connect(&self, address: String) {
        let self_sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                let ws_stream = match connect_async(address.clone()).await {
                    Ok((mut stream, _response)) => {
                        tracing::info!("Connected to {}", address);
                        let settings = &mut *gRelaySettings.lock().await;
                        let mut m = SdInnerMessage::default();
                        m.server_id = settings.server_id.clone();
                        m.server_type = SdServerType::KSdRelayServer as i32;
                        m.msg_type = i32::from(SdInnerMessageType::KSdInnerHello);
                        m.hello = Some(SdInnerHello {
                            server_name: settings.server_name.clone(),
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
                                let settings = &mut *gRelaySettings.lock().await;
                                let mut m = SdInnerMessage::default();
                                m.server_id = settings.server_id.clone();
                                m.server_type = SdServerType::KSdRelayServer as i32;
                                m.msg_type = i32::from(SdInnerMessageType::KSdInnerHeartBeat);
                                m.heartbeat = Some(SdInnerHeartBeat {
                                    server_name: "".to_string(),
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