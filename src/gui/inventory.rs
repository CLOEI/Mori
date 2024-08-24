use crate::manager::Manager;
use eframe::egui::{self, Ui};
use crate::utils;

#[derive(Default)]
pub struct Inventory {
    pub selected_bot: String,
}

impl Inventory {
    pub fn render(&mut self, ui: &mut Ui, manager: &mut Manager, _ctx: &egui::Context) {
        self.selected_bot = utils::config::get_selected_bot();
        if !self.selected_bot.is_empty() {
            if let Some(bot) = manager.get_bot(&self.selected_bot) {
                let inventory_items = {
                    let inventory = bot.inventory.read();
                    inventory.items.clone()
                };

                ui.label("Inventory");
                ui.separator();

                for inventory_item in inventory_items {
                    let item = manager.items_database.get_item(&(inventory_item.id as u32)).unwrap();
                    ui.horizontal(|ui| {
                        ui.label(item.name.clone());
                        ui.label(format!("x{}", inventory_item.amount));
                    });
                }
            }
        }
    }
}

