use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use crate::{gRelayConnMgr, gRelaySettings, gRoomMgr};
use crate::relay_conn_mgr::RelayConnManager;
use crate::relay_room::RelayRoom;
use crate::relay_room_mgr::RelayRoomManager;

pub struct RelayContext {

}

impl RelayContext {
    pub fn new() -> RelayContext {
        RelayContext {
        }
    }

    pub fn init(&mut self) -> bool {
        true
    }
}