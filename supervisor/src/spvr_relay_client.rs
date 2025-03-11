use protocol::grpc_base::{HeartBeatReply, HeartBeatRequest};
use protocol::grpc_relay::grpc_relay_client::GrpcRelayClient;
use protocol::grpc_relay::grpc_relay_server::GrpcRelay;
use protocol::grpc_relay::{RelayStreamRequest};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use tokio_stream::Stream;
use tonic::client::Grpc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Response, Status, Streaming};
use crate::{gSpvrRelayClients, gSpvrSettings};
use crate::spvr_settings::RelayServerConfig;

pub struct SpvrRelayClient {
    pub client: Arc<Mutex<Option<GrpcRelayClient<Channel>>>>,
    pub hb_index: i64,
    pub config: RelayServerConfig,
}

async fn echo_requests_iter() -> impl Stream<Item = RelayStreamRequest> {
    let server_id = gSpvrSettings.lock().await.server_id.clone();
    tokio_stream::iter(1..usize::MAX).map(move |i| RelayStreamRequest {
        server_id: server_id.clone(),
        message: format!("msg {:02}", i),
    })
}

impl SpvrRelayClient {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            hb_index: 0,
            config: RelayServerConfig::default(),
        }
    }

    pub async fn connect(&mut self, config: RelayServerConfig) -> bool {
        self.config = config.clone();
        let addr = format!("http://{}:{}", config.ip, config.port);
        tracing::info!("connecting to {}", addr);
        let conn = GrpcRelayClient::connect(addr).await;
        if let Err(e) = conn {
            tracing::error!("connect grpc remote error: {}", e);
            return false;
        }
        let conn = conn.unwrap();
        self.client = Arc::new(Mutex::new(Some(conn)));
        true
    }

    pub async fn heartbeat(&mut self) -> bool {
        let server_id = gSpvrSettings.lock().await.server_id.clone();
        if let Some(client) = self.client.lock().await.as_mut() {
            let r = client.heart_beat(tonic::Request::new(HeartBeatRequest {
                server_id,
                hb_index: self.hb_index,
            })).await;

            if let Ok(r) = r {
                tracing::info!("heart beat : {}", self.hb_index);
                self.hb_index += 1;
                return true;
            }
        }
        self.client = Arc::new(Mutex::new(None));
        false
    }

    pub async fn guard(relay_client: Arc<Mutex<SpvrRelayClient>>) {
        //self.scheduler = JobScheduler::new().await.unwrap();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                tracing::info!("will check heartbeat...");
                let relay_client = relay_client.clone();
                // 如果这里是&，则下面relay_client又会引用自己，导致死锁
                let config = relay_client.lock().await.config.clone();
                if relay_client.lock().await.heartbeat().await {
                    tracing::info!("connection is ok: {:?}", Instant::now());
                    continue;
                } else {
                    tracing::error!("connection is closed, will retry.");
                    relay_client.lock().await.connect(config).await;
                }
            }
        });
    }

    pub async fn start_streaming_request(&mut self, num: usize) {
        let in_stream = echo_requests_iter().await.take(num);

        if let Some(client) = &mut *self.client.lock().await {
            let response = client
                .stream_request(in_stream).await;
            if let Err(e) = response {
                tracing::error!("streaming request error: {}", e);
                return;
            }
            let response = response.unwrap();

            tokio::spawn(async move {
                let mut resp_stream = response.into_inner();
                while let Some(received) = resp_stream.next().await {
                    let received = received.unwrap();
                    println!("\treceived message: `{}`", received.message);
                }
            });
        }

    }
}