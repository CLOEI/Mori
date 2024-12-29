use crate::manager::bot_manager::BotManager;
use crate::utils;
use eframe::egui::{self, Ui};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Growscan {
    pub selected_bot: String,
}

impl Growscan {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<BotManager>>) {
        self.selected_bot = utils::config::get_selected_bot();
        if !self.selected_bot.is_empty() {
            let bot = {
                let manager = manager.read().unwrap();
                manager.get_bot(&self.selected_bot).cloned()
            };
            if let Some(bot) = bot {
                ui.allocate_ui(ui.available_size(), |ui| {
                    ui.vertical(|ui| {
                        let (objects, tiles) = {
                            let world = bot.world.read().unwrap();
                            (world.dropped.clone(), world.tiles.clone())
                        };

                        egui::ScrollArea::vertical()
                            .id_source("objects_scroll")
                            .show(ui, |ui| {
                                ui.heading("Objects");

                                let mut item_amounts: HashMap<u32, u64> = HashMap::new();
                                for item in &objects.items {
                                    *item_amounts.entry(item.id.into()).or_insert(0) += item.count as u64;
                                }
                                

                                let mut sorted_items: Vec<_> = item_amounts.iter().collect();
                                sorted_items.sort_by_key(|&(id, _)| id);

                                egui::Grid::new("objects_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 10.0])
                                    .striped(true)
                                    .min_col_width(ui.available_width())
                                    .show(ui, |ui| {
                                        for (id, count) in sorted_items {
                                            if let Some(item_data) = bot
                                                .item_database
                                                .read()
                                                .unwrap()
                                                .get_item(&(*id as u32))
                                            {
                                                let item_name = &item_data.name;
                                                ui.label(format!("{} {}", count, item_name));
                                                ui.end_row();
                                            }
                                        }
                                    });
                            });

                        ui.add_space(20.0);

                        egui::ScrollArea::vertical()
                            .id_source("tiles_scroll")
                            .show(ui, |ui| {
                                ui.heading("Tiles");

                                let mut item_counts = HashMap::new();
                                for tile in &tiles {
                                    if tile.foreground_item_id != 0 {
                                        *item_counts.entry(tile.foreground_item_id).or_insert(0) +=
                                            1;
                                    }
                                    if tile.background_item_id != 0 {
                                        *item_counts.entry(tile.background_item_id).or_insert(0) +=
                                            1;
                                    }
                                }

                                let mut sorted_tiles: Vec<_> = item_counts.iter().collect();
                                sorted_tiles.sort_by_key(|&(id, _)| id);

                                egui::Grid::new("tiles_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 10.0])
                                    .striped(true)
                                    .min_col_width(ui.available_width())
                                    .show(ui, |ui| {
                                        for (id, count) in sorted_tiles {
                                            if let Some(item_data) = bot
                                                .item_database
                                                .read()
                                                .unwrap()
                                                .get_item(&(*id as u32))
                                            {
                                                let item_name = &item_data.name;
                                                ui.label(format!("{} {}", count, item_name));
                                                ui.end_row();
                                            }
                                        }
                                    });
                            });
                    });
                });
            }
        }
    }
}
