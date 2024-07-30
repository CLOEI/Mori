use std::fs;

use eframe::egui::{self};

use crate::{manager::Manager, types::e_login_method::ELoginMethod, App, Bot, Data};

#[derive(Default)]
pub struct AddBotDialog {
    pub username: String,
    pub password: String,
    pub code: String,
    pub method: ELoginMethod,
    pub open: bool,
}

impl AddBotDialog {
    pub fn render(&mut self, manager: &mut Manager, ctx: &egui::Context) {
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
                                        ELoginMethod::UBISOFT,
                                        "UBISOFT",
                                    );
                                });
                            ui.end_row();
                        });
                    if ui.button("Add").clicked() {
                        manager.add_bot(
                            self.username.clone(),
                            self.password.clone(),
                            self.code.clone(),
                            self.method.clone(),
                        );
                        let mut data =
                            serde_json::from_str::<Data>(&fs::read_to_string("data.json").unwrap())
                                .unwrap();
                        data.bots.push(Bot {
                            username: self.username.clone(),
                            password: self.password.clone(),
                            code: self.code.clone(),
                            method: self.method.clone(),
                        });
                        fs::write("data.json", &serde_json::to_string_pretty(&data).unwrap())
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
