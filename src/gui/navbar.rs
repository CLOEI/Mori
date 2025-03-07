// use super::add_bot_dialog::AddBotDialog;
// use crate::utils;
// use crate::{manager::bot_manager::BotManager, texture_manager::TextureManager};
// use eframe::egui::{self, include_image, Ui};
// use std::sync::{Arc, RwLock};
use eframe::egui::{self, Ui};

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
    pub fn render(
        &mut self,
        ui: &mut Ui,
        // ui: &mut egui::Ui,
        // add_bot_dialog: &mut AddBotDialog,
        // manager: &Arc<RwLock<BotManager>>,
    ) {
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(
                    egui_phosphor::variants::fill::USER.to_owned() + " Bots",
                ))
                .clicked()
            {
                self.current_menu = "bots".to_string();
            }
            if ui
                .add(egui::Button::new(egui::RichText::new(
                    egui_phosphor::variants::fill::DATABASE.to_owned() + " Item database",
                )))
                .clicked()
            {
                self.current_menu = "item_database".to_string();
            }
            if ui
                .add(egui::Button::new(
                    egui_phosphor::variants::fill::GPS.to_owned() + " Proxy",
                ))
                .clicked()
            {
                self.current_menu = "proxy_list".to_string();
            }
            if ui
                .add(egui::Button::new(
                    egui_phosphor::variants::fill::GEAR_SIX.to_owned() + " Settings",
                ))
                .clicked()
            {
                self.current_menu = "settings".to_string();
            }
            // ui.separator();
            // if ui
            //     .add(egui::Button::new(
            //         egui_phosphor::variants::fill::PLUS.to_owned() + " Add bot",
            //     ))
            //     .clicked()
            // {
            //     add_bot_dialog.open = true;
            // }
            // if ui
            //     .add(egui::Button::new(
            //         egui_phosphor::variants::fill::MINUS.to_owned() + "  Remove bot",
            //     ))
            //     .clicked()
            // {
            //     let selected_bot = utils::config::get_selected_bot();
            //     manager.write().unwrap().remove_bot(&selected_bot);
            // }
        });
    }
}
