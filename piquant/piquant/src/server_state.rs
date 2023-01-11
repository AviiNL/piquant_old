use std::collections::BTreeMap;

use valence::{
    prelude::{InventoryId, PlayerListId},
    protocol::BlockPos,
};

pub struct ServerState {
    pub player_lists: Option<PlayerListId>,
    pub inventories: BTreeMap<BlockPos, InventoryId>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            player_lists: None,
            inventories: BTreeMap::new(),
        }
    }
}
