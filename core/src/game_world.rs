use crate::types::player::Player;
use gtitem_r::structs::ItemDatabase;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug)]
pub struct GameWorld {
    pub data: Mutex<gtworld_r::World>,
    pub players: Mutex<HashMap<u32, Player>>,
    pub item_database: Arc<RwLock<ItemDatabase>>,
}

impl GameWorld {
    pub fn new(item_database: Arc<RwLock<ItemDatabase>>) -> Self {
        Self {
            data: Mutex::new(gtworld_r::World::new()),
            players: Mutex::new(HashMap::new()),
            item_database,
        }
    }
}
