use std::sync::{Arc, RwLock};
use std::thread::spawn;
use crate::manager::bot_manager::BotManager;
use eframe::egui::{self, Ui};
use crate::bot::{drop_item, trash_item};
use crate::utils;

#[derive(Default)]
pub struct Inventory {
    pub selected_bot: String,
}

impl Inventory {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<BotManager>>) {
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

                ui.vertical(|ui| {
                    for (id, inventory_item) in inventory_items {
                        let item = {
                            let item = manager.read().unwrap().items_database.get_item(&(id as u32)).unwrap();
                            item.clone()
                        };
                        ui.horizontal(|ui| {
                            ui.label(item.name.clone());
                            ui.label(format!("x{}", inventory_item.amount));
                            ui.group(|ui| {
                                if ui.button("Drop").clicked() {
                                    let bot_clone = bot.clone();
                                    spawn(move || {
                                        drop_item(&bot_clone, id as u32,  1);
                                    });
                                }
                                if ui.button("Trash").clicked() {
                                    let bot_clone = bot.clone();
                                    spawn(move || {
                                        trash_item(&bot_clone, id as u32,  1);
                                    });
                                }
                            });
                        });
                    }
                });
            }
        }
    }
}

