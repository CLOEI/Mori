use crate::manager::bot_manager::BotManager;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct ItemDatabase {
    search_query: String,
    selected_item_index: Option<u32>,
}

impl ItemDatabase {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<BotManager>>, _ctx: &egui::Context) {
        let manager = manager.read().unwrap();
        let items_database = manager.items_database.read().unwrap();
    }
}
