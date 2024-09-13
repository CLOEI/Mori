use std::sync::{Arc, RwLock};
use eframe::egui::{self, include_image, Ui};
use crate::manager::Manager;
use crate::utils;
use super::add_bot_dialog::AddBotDialog;

pub struct Navbar {
    pub current_menu: String,
}

impl Default for Navbar {
    fn default() -> Self {
        Navbar {
            current_menu: "bots".to_string(),
        }
    }
}

impl Navbar {
    pub fn render(&mut self, ui: &mut egui::Ui, add_bot_dialog: &mut AddBotDialog, manager: &Arc<RwLock<Manager>>) {
        ui.horizontal(|ui| {
            ui.heading("Mori");
            ui.separator();
            if ui
                .add(egui::Button::image_and_text(
                    egui::include_image!("../../assets/bot.png"),
                    "Bots",
                ))
                .clicked()
            {
                self.current_menu = "bots".to_string();
            }
            if ui
                .add(egui::Button::image_and_text(
                    include_image!("../../assets/database.png"),
                    "Item database",
                ))
                .clicked()
            {
                self.current_menu = "item_database".to_string();
            }
            if ui
                .add(egui::Button::image_and_text(
                    include_image!("../../assets/settings.png"),
                    "Settings",
                ))
                .clicked()
            {
                self.current_menu = "settings".to_string();
            }
            ui.separator();
            if ui
                .add(egui::Button::image_and_text(
                    include_image!("../../assets/plus.png"),
                    "Add bot",
                ))
                .clicked()
            {
                add_bot_dialog.open = true;
            }
            if ui
                .add(egui::Button::image_and_text(
                    include_image!("../../assets/minus.png"),
                    "Remove bot",
                ))
                .clicked()
            {
                let selected_bot = utils::config::get_selected_bot();
                manager.write().unwrap().remove_bot(&selected_bot);
            }
        });
    }
}
