use valence::{prelude::EntityId, protocol::ItemStack};

pub struct ClientState {
    pub entity_id: EntityId,
    pub held_item_slot: i16,
    pub creative_mode_slots: Vec<Option<ItemStack>>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            entity_id: Default::default(),
            held_item_slot: 0,
            creative_mode_slots: Vec::new(),
        }
    }
}
