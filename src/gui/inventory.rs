use std::sync::{Arc, RwLock};
use crate::manager::Manager;
use eframe::egui::{self, Ui};
use crate::utils;

#[derive(Default)]
pub struct Inventory {
    pub selected_bot: String,
}

impl Inventory {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<Manager>>, _ctx: &egui::Context) {
        self.selected_bot = utils::config::get_selected_bot();
        if !self.selected_bot.is_empty() {
            let bot = {
                let manager = manager.read().unwrap();

                match manager.get_bot(&self.selected_bot) {
                    Some(bot) => Some(bot.clone()),
                    None => None,
                }
            };
            if let Some(bot) = bot {
                let inventory_items = {
                    let inventory = bot.inventory.read().unwrap();
                    inventory.items.clone()
                };

                ui.label("Inventory");
                ui.separator();

                for inventory_item in inventory_items {
                    let item = {
                        let item = manager.read().unwrap().items_database.get_item(&(inventory_item.id as u32)).unwrap();
                        item.clone()
                    };
                    ui.horizontal(|ui| {
                        ui.label(item.name.clone());
                        ui.label(format!("x{}", inventory_item.amount));
                    });
                }
            }
        }
    }
}

