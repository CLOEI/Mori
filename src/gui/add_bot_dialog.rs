use crate::{
    manager::bot_manager::BotManager,
    types::{config::BotConfig, elogin_method::ELoginMethod},
    utils,
};
use eframe::egui::{self};
use std::fs;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AddBotDialog {
    pub username: String,
    pub password: String,
    pub steam_user: String,
    pub steam_pass: String,
    pub code: String,
    pub method: ELoginMethod,
    pub use_proxy: bool,
    pub open: bool,
}

impl AddBotDialog {
    pub fn render(&mut self, manager: &Arc<RwLock<BotManager>>, ctx: &egui::Context) {
        if self.open {
            let mut close_dialog = false;
            egui::Window::new("Add bot")
                .resizable(false)
                .open(&mut self.open)
                .show(ctx, |ui| {
                    egui::Grid::new("add_bot_grid")
                        .min_col_width(100.0)
                        .max_col_width(100.0)
                        .show(ui, |ui| {
                            ui.label("Username");
                            ui.text_edit_singleline(&mut self.username);
                            ui.end_row();
                            ui.label("Password");
                            ui.text_edit_singleline(&mut self.password);
                            ui.end_row();
                            ui.label("Steam username");
                            ui.text_edit_singleline(&mut self.steam_user);
                            ui.end_row();
                            ui.label("Steam Password");
                            ui.text_edit_singleline(&mut self.steam_pass);
                            ui.end_row();
                            ui.label("2FA Code");
                            ui.text_edit_singleline(&mut self.code);
                            ui.end_row();
                            ui.label("Login Method");
                            egui::ComboBox::from_id_source("login_method")
                                .selected_text(format!("{:?}", self.method))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.method,
                                        ELoginMethod::LEGACY,
                                        "LEGACY",
                                    );
                                    ui.selectable_value(
                                        &mut self.method,
                                        ELoginMethod::GOOGLE,
                                        "GOOGLE",
                                    );
                                    ui.selectable_value(
                                        &mut self.method,
                                        ELoginMethod::APPLE,
                                        "APPLE",
                                    );
                                    ui.selectable_value(
                                        &mut self.method,
                                        ELoginMethod::STEAM,
                                        "STEAM",
                                    );
                                });
                            ui.end_row();
                            ui.checkbox(&mut self.use_proxy, "Use proxy");
                        });
                    if ui.button("Add").clicked() {
                        let config;
                        if self.method == ELoginMethod::STEAM {
                            config = BotConfig {
                                payload: format!(
                                    "{}|{}|{}|{}",
                                    self.username, self.password, self.steam_user, self.steam_pass
                                ),
                                recovery_code: self.code.clone(),
                                login_method: self.method.clone(),
                                token: "".to_string(),
                                data: "".to_string(),
                                use_proxy: self.use_proxy,
                            };
                        } else {
                            config = BotConfig {
                                payload: format!("{}|{}", self.username, self.password),
                                recovery_code: self.code.clone(),
                                login_method: self.method.clone(),
                                token: "".to_string(),
                                data: "".to_string(),
                                use_proxy: self.use_proxy,
                            };
                        }
                        {
                            manager.write().unwrap().add_bot(config.clone());
                        }
                        let mut data = utils::config::parse_config().unwrap();
                        data.bots.push(config);
                        fs::write("config.json", &serde_json::to_string_pretty(&data).unwrap())
                            .unwrap();
                        self.username.clear();
                        self.password.clear();
                        self.code.clear();
                        self.method = ELoginMethod::LEGACY;
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.open = false;
            }
        }
    }
}
