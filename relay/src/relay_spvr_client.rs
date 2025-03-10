use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_stream::Stream;
use tonic::client::Grpc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint};
use protocol::grpc_relay::grpc_relay_client::GrpcRelayClient;
use protocol::grpc_base::HeartBeatRequest;
use protocol::grpc_relay::EchoRequest;

pub struct RelaySpvrClient {
    pub client: Arc<Mutex<Option<GrpcRelayClient<Channel>>>>,
    pub hb_index: i64,
}

fn echo_requests_iter() -> impl Stream<Item = EchoRequest> {
    tokio_stream::iter(1..usize::MAX).map(|i| EchoRequest {
        message: format!("msg {:02}", i),
    })
}

impl RelaySpvrClient {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            hb_index: 0,
        }
    }

    pub async fn connect(&mut self) -> bool {
        let addr = "http://127.0.0.1:50051";
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
        if let Some(client) = self.client.lock().await.as_mut() {
            let r = client.heart_beat(tonic::Request::new(HeartBeatRequest {
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

    pub async fn guard(client: Arc<Mutex<RelaySpvrClient>>) {
        //self.scheduler = JobScheduler::new().await.unwrap();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                let client = client.clone();
                interval.tick().await;
                if client.lock().await.heartbeat().await {
                    tracing::info!("connection is ok: {:?}", Instant::now());
                    continue;
                } else {
                    tracing::error!("connection is closed, will retry.");
                    client.lock().await.connect().await;
                }
            }
        });
    }

    pub async fn send_message(&self, msg: String) {
        // let request = tonic::Request::new(HelloRequest {
        //     name: "xx--===".to_string(),
        // });
        //
        // if let Some(client) = &mut *self.client.lock().await {
        //     let response = client.say_hello(request).await;
        //     if let Err(e) = response {
        //         tracing::error!("say hello error: {}", e);
        //         return;
        //     }
        //     let response = response.unwrap();
        //     tracing::info!("resp: {}", response.into_inner().message);
        // }
    }

    async fn bidirectional_streaming_echo(&self, num: usize) {
        let in_stream = echo_requests_iter().take(num);

        // if let Some(client) = &mut *self.client.lock().await {
        //
        // }
        //
        // let response = self.client.lock().await.
        //     .bidirectional_streaming_echo(in_stream)
        //     .await
        //     .unwrap();
        //
        // let mut resp_stream = response.into_inner();
        //
        // while let Some(received) = resp_stream.next().await {
        //     let received = received.unwrap();
        //     println!("\treceived message: `{}`", received.message);
        // }
    }
}