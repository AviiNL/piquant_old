use std::collections::BTreeMap;

use valence::{
    prelude::{InventoryId, PlayerListId},
    protocol::BlockPos,
};

use crate::chat::MessageQueue;

pub struct ServerState {
    pub player_lists: Option<PlayerListId>,
    pub inventories: BTreeMap<BlockPos, InventoryId>,
    pub message_queue: MessageQueue,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            player_lists: None,
            inventories: BTreeMap::new(),
            message_queue: MessageQueue::new(),
        }
    }
}
