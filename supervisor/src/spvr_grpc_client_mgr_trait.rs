use protocol::spvr_inner::{SpvrInnerHeartBeat, SpvrInnerHello};

pub trait SpvrGrpcClientManager {
    // hello
    async fn on_hello(&self, server_id: String, msg: SpvrInnerHello);

    // heart beat
    async fn on_heartbeat(&self, server_id: String, msg: SpvrInnerHeartBeat);

    // closed
    async fn on_close(&self, server_id: String);
}