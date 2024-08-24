use eframe::egui::{self, include_image, Ui};

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
    pub fn render(&mut self, ui: &mut egui::Ui, add_bot_dialog: &mut AddBotDialog) {
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
                    include_image!("../../assets/earth.png"),
                    "World",
                ))
                .clicked()
            {
                self.current_menu = "world_map".to_string();
            }
            if ui
                .add(egui::Button::image_and_text(
                    include_image!("../../assets/backpack.png"),
                    "Inventory",
                ))
                .clicked()
            {
                self.current_menu = "inventory".to_string();
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
                    include_image!("../../assets/blocks.png"),
                    "Features",
                ))
                .clicked()
            {
                self.current_menu = "features".to_string();
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
        });
    }
}
