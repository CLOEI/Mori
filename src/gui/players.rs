use crate::manager::bot_manager::BotManager;
use crate::utils;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};

#[derive(Default, Debug, Clone)]
pub struct PlayersScan {
    pub selected_bot: String,
    pub search_text: String
}
impl PlayersScan {
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
                        let playersz = {
                            let players = bot.players.lock().unwrap();
                            players.clone()
                        };
                        ui.horizontal(|ui| {
                            ui.label(format!("Players in world: {}", playersz.len()));
                            ui.add_space(10.0);
                            ui.separator();
                            ui.label("Search Players:");
                            ui.text_edit_singleline(&mut self.search_text);
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_source("players_scroll")
                            .show(ui, |ui| {
                                ui.heading("Players");
                                ui.separator();
                                egui::Grid::new("players_grid")
                                    .num_columns(3)
                                    .spacing([10.0, 10.0])
                                    .striped(true)
                                    .min_col_width(ui.available_width() / 3.0)
                                    .show(ui, |ui| {
                                        for player in &playersz {
                                            let formatted_name = self.format_name(&player.name);
                                            let search_text_lower = self.search_text.to_lowercase();
                                            let player_name_lower = player.name.to_lowercase();
    
                                            if self.search_text.is_empty() || player_name_lower.contains(&search_text_lower) {
                                                if self.search_text.is_empty() || formatted_name.to_lowercase().contains(&search_text_lower) {
                                                    if player.name.contains("w") {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.colored_label(egui::Color32::from_rgb(255, 255, 255), format!("{}", formatted_name));
                                                            let position_x = player.position.x as i32 / 32;
                                                            let position_y = player.position.y as i32 / 32;
                                                            ui.label(format!("Position: ({}, {})", position_x, position_y));
                                                            ui.label(format!("Country: {}", player.country));
                                                        });
                                                    } else if player.name.contains("2") {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.colored_label(egui::Color32::from_rgb(0, 255, 0), format!("{} (owner)", formatted_name));
                                                            let position_x = player.position.x as i32 / 32;
                                                            let position_y = player.position.y as i32 / 32;
                                                            ui.label(format!("Position: ({}, {})", position_x, position_y));
                                                            ui.label(format!("Country: {}", player.country));
                                                        });
                                                    } else if player.name.contains("4") {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Name:");
                                                            ui.colored_label(egui::Color32::from_rgb(252, 28, 0), format!("{}", formatted_name));
                                                            let position_x = player.position.x as i32 / 32;
                                                            let position_y = player.position.y as i32 / 32;
                                                            ui.label(format!("Position: ({}, {})", position_x, position_y));
                                                            ui.label(format!("Country: {}", player.country));
                                                        });
                                                    } else if player.name.contains("#") {
                                                        ui.label("Name: Moderator");
                                                    } else {
                                                        ui.label(format!("Name: {}", formatted_name));
                                                    }
                                                    ui.end_row();
                                                }
                                            }
                                        }
                                    });
                            });
                    });
                });
            }
        }
    }
    
    fn format_name(&self, name: &str) -> String {
        let mut formatted_name = name.to_string();
        formatted_name = formatted_name.replace("``", "");
        formatted_name = formatted_name.replace("`w", "");
        formatted_name = formatted_name.replace("`2", "");
        formatted_name
    }
}