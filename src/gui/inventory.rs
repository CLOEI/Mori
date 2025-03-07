use crate::manager::bot_manager::BotManager;
use crate::utils;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};
use std::thread::spawn;

#[derive(Default)]
pub struct Inventory {
    pub selected_bot: String,
    pub searched_word: String,
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
                    let inventory = bot.inventory.lock().unwrap();
                    inventory.items.clone()
                };
                let inventory_capacity = inventory_items.capacity();
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Inventory Size: ");
                        ui.label(inventory_capacity.to_string());
                        ui.separator();
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.searched_word);
                    });
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("inventory_grid")
                            .num_columns(2)
                            .spacing([0.0, 20.0])
                            .striped(true)
                            .min_col_width(ui.available_width() / 2.0)
                            .show(ui, |ui| {
                                for (id, inventory_item) in inventory_items {
                                    let (item, wear_disabled) = {
                                        let item = manager
                                            .read()
                                            .unwrap()
                                            .items_database
                                            .read()
                                            .unwrap()
                                            .get_item(&(id as u32))
                                            .unwrap();
                                        (item.clone(), item.action_type != 20)
                                    };
                                    if self.searched_word.is_empty() || item.name.to_lowercase().contains(&self.searched_word.to_lowercase()) {
                                        ui.horizontal(|ui| {
                                            ui.label(item.name.clone());
                                            ui.label(format!("x{}", inventory_item.amount));
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .add_enabled(
                                                        !wear_disabled,
                                                        egui::Button::new("Wear"),
                                                    )
                                                    .clicked()
                                                {
                                                    let bot_clone = bot.clone();
                                                    spawn(move || {
                                                        bot_clone.wear(id as u32);
                                                    });
                                                }
                                                if ui.button("Drop All").clicked() {
                                                    let bot_clone = bot.clone();
                                                    spawn(move || {
                                                        bot_clone.drop_item(id as u32, inventory_item.amount as u32);
                                                    });
                                                }
                                                if ui.button("Drop").clicked() {
                                                    let bot_clone = bot.clone();
                                                    spawn(move || {
                                                        bot_clone.drop_item(id as u32, 1);
                                                    });
                                                }
                                                if ui.button("Trash").clicked() {
                                                    let bot_clone = bot.clone();
                                                    spawn(move || {
                                                        bot_clone.trash_item(id as u32, 1);
                                                    });
                                                }
                                            },
                                        );
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });
            }
        }
    }
}
