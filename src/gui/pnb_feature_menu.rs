use crate::manager::bot_manager::BotManager;
use crate::utils::logging::{info, warn};
use crate::core::features::auto_pnb::AutoPNB;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;

impl AutoPNB {
    pub fn render(&mut self, ui: &mut Ui, manager: Arc<RwLock<BotManager>>) {
        self.selected_bot = crate::utils::config::get_selected_bot();

        if self.selected_bot.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Please select a bot.");
            });
            return;
        }

        ui.vertical(|ui| {
            ui.heading("PNB");

            ui.horizontal(|ui| {
                if self.is_farming {
                    if ui.button("Pause").clicked() {
                        self.pause_autofarm();
                    }
                } else {
                    if ui.button("Start").clicked() {
                        self.start_autofarm(manager.clone());
                    }
                }

                ui.label(if self.is_farming {
                    "PNB is running..."
                } else if self.is_paused.load(Ordering::SeqCst) {
                    "PNB is paused."
                } else {
                    "PNB is idle."
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Enter Item ID:");
                if ui
                    .add(egui::TextEdit::singleline(&mut self.temp_item_id).hint_text("Item ID"))
                    .lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    if let Ok(parsed_id) = self.temp_item_id.parse::<u32>() {
                        self.item_id = parsed_id;
                    } else {
                        ui.label("Invalid Item ID.");
                    }
                }
            });

            ui.separator();
            ui.heading("Select Positions (5x5 Grid):");

            ui.vertical_centered(|ui| {
                for row in 0..5 {
                    ui.horizontal(|ui| {
                        for col in 0..5 {
                            let x = col as i32 - 2;
                            let y = row as i32 - 2;

                            if row == 2 && col == 2 {
                                ui.label(" O ");
                            } else {
                                if ui.checkbox(&mut self.grid_selection[row][col], "").clicked() {
                                    let position = (x, y);
                                    if self.grid_selection[row][col] {
                                        self.selected_positions.push(position);
                                    } else {
                                        self.selected_positions.retain(|&pos| pos != position);
                                    }
                                }
                            }
                        }
                    });
                }
            });
        });
    }
}
