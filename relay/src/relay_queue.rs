use std::sync::Arc;
use axum::body::Bytes;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use crate::relay_conn::RelayConn;
use crate::relay_room::RelayRoom;

pub struct RelayPacket {
    // pub conn: Arc<Mutex<RelayConn>>,
    pub except_id: String,
    pub room: RelayRoom,
    pub payload: Bytes,
    pub relay_msg_index: i64,
}

pub struct RelayQueue {
    pub room_id: String,
    pub pkt_sender: Sender<RelayPacket>,
    pub pkt_receiver: Arc<Mutex<Receiver<RelayPacket>>>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl RelayQueue {
    pub fn new(room_id: String) -> Self {
        let (sender, receiver) = mpsc::channel::<RelayPacket>(1024);
        RelayQueue {
            room_id,
            pkt_sender: sender,
            pkt_receiver: Arc::new(Mutex::new(receiver)),
            shutdown: Arc::new(Default::default()),
        }
    }

    pub async fn run(&mut self) {
        let receiver = self.pkt_receiver.clone();
        let room_id = self.room_id.clone();
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            loop {
                let mut guard = receiver.lock().await;
                let shutdown = shutdown.clone();
                tokio::select! {
                    pkt = guard.recv() => {
                       match pkt {
                            Some(pkt) => {
                                let payload = pkt.payload;
                                let relay_msg_index = pkt.relay_msg_index;
                                let except_id = pkt.except_id;
                                //pkt.conn.lock().await.send_bin_message_with_index(relay_msg_index, payload).await;
                                pkt.room.notify_except(except_id, relay_msg_index, payload).await;
                            },
                            None => {

                            }
                        }
                    },
                    _ = shutdown.notified() => {
                        tracing::warn!("exit relay queue notified.");
                        break;
                    }
                }
            }
            tracing::warn!("relay queue exit for room: {}", room_id);
        });
    }

    pub async fn send(&self, pkt: RelayPacket) {
        if let Err(e) = self.pkt_sender.send(pkt).await {
            tracing::error!("error sending relay message to queue: {e}");
        }
    }

    pub async fn exit(&self) {
        self.shutdown.notify_one();
        tracing::warn!("call shutdown for room: {}", self.room_id);
    }
}