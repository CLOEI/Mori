use crate::manager::bot_manager::BotManager;
use crate::core::features::auto_spam::Autospamv1;
use eframe::egui::{self, Ui};
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;

impl Autospamv1 {
    pub fn render(&mut self, ui: &mut Ui, manager: Arc<RwLock<BotManager>>) {
        self.selected_bot = crate::utils::config::get_selected_bot();

        if self.selected_bot.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Please select a bot.");
            });
            return;
        }

        ui.vertical(|ui| {
            ui.heading("Auto Spam - module version: 1.0.0");

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
            });

            ui.separator();

            // Hedef pozisyonları
            ui.heading("Target Positions:");
            ui.horizontal(|ui| {
                if ui.button("Add Position").clicked() {
                    self.selected_xy_of_world.push((0, 0)); 
                }
                if ui.button("Remove Position").clicked() {
                    self.selected_xy_of_world.pop();
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
