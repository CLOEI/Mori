use crate::core::features::auto_spam::AutoSpam;
use crate::manager::bot_manager::BotManager;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;

impl AutoSpam {
    pub fn render(&mut self, ui: &mut Ui, manager: Arc<RwLock<BotManager>>) {
        self.selected_bot = crate::utils::config::get_selected_bot();

        if self.selected_bot.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Please select a bot.");
            });
            return;
        }
        ui.vertical(|ui| {
            ui.heading("Spam");

            ui.horizontal(|ui| {
                if self.is_talking {
                    if ui.button("Pause Spam").clicked() {
                        self.pause_autospam();
                    }
                } else {
                    if ui.button("Start Spam").clicked() {
                        self.start_autospam(manager.clone());
                    }
                }

                ui.label(if self.is_talking {
                    "Spam is running..."
                } else if self.is_paused.load(Ordering::SeqCst) {
                    "Spam is paused."
                } else {
                    "Spam is idle."
                });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Spam Message:");
                ui.add(egui::TextEdit::singleline(&mut self.temp_spam_text).hint_text("Enter message"));
                let mut checkbox = egui::Checkbox::new(&mut self.is_anti_positive_enabled, "Enable AI Power");
                checkbox = checkbox.indeterminate(self.is_enabled);
                ui.add(checkbox);
            });

            ui.separator();

            ui.heading("Target Positions:");
            let mut x_input = self.temp_x_input.clone();
            let mut y_input = self.temp_y_input.clone();
            
            ui.horizontal(|ui| {
                ui.label("X:");
                if ui.add(egui::TextEdit::singleline(&mut x_input).hint_text("X coordinate")).changed() {
                    self.temp_x_input = x_input.clone();
                }
            
                ui.label("Y:");
                if ui.add(egui::TextEdit::singleline(&mut y_input).hint_text("Y coordinate")).changed() {
                    self.temp_y_input = y_input.clone();
                }
            
                if ui.button("Add Position").clicked() {
                    if let (Ok(x), Ok(y)) = (x_input.parse::<i32>(), y_input.parse::<i32>()) {
                        self.add_position(x, y);
                        self.temp_x_input.clear();
                        self.temp_y_input.clear();
                    } else {
                        println!("Invalid input for coordinates.");
                    }
                }
            });
            ui.separator();
            if !self.selected_xy_of_world.is_empty() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        for (x, y) in &self.selected_xy_of_world {
                            ui.label(format!("Position: ({}, {})", x, y));
                        }
                    });
                });
            }

            ui.separator();

            ui.label("Status: ");
            ui.label(if self.is_talking {
                "Spam is running..."
            } else if self.is_paused.load(Ordering::SeqCst) {
                "Spam is paused."
            } else {
                "Spam is idle."
            });
        });
    }
}