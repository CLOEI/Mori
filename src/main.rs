mod bot;
mod manager;
mod types;
mod utils;

use std::fs;

use eframe::egui::{self, ViewportBuilder};
use manager::Manager;
use serde::{Deserialize, Serialize};
use types::e_login_method::ELoginMethod;

#[derive(Serialize, Deserialize)]
struct Data {
    bots: Vec<Bot>,
}

#[derive(Serialize, Deserialize)]
struct Bot {
    username: String,
    password: String,
    code: String,
    method: ELoginMethod,
}

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: ViewportBuilder {
            inner_size: Some(egui::vec2(800.0, 400.0)),
            resizable: Some(false),
            ..Default::default()
        },
        ..Default::default()
    };
    let _ = eframe::run_native("Mori", options, Box::new(|cc| Ok(Box::new(App::new(cc)))));
}
struct App {
    current_menu: String,
    search_query: String,
    manager: Manager,
    new_bot_username: String,
    new_bot_password: String,
    new_bot_code: String,
    new_bot_method: ELoginMethod,
    show_add_bot_dialog: bool,
    bots: Vec<Bot>,
    selected_bot_name: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut manager = Manager::new().unwrap();

        let data = match fs::read_to_string("data.json") {
            Ok(data) => data,
            Err(_) => {
                let data = Data { bots: vec![] };
                let json = serde_json::to_string_pretty(&data).unwrap();
                fs::write("data.json", &json).unwrap();
                json
            }
        };
        let json = serde_json::from_str::<Data>(&data).unwrap();
        for bot in &json.bots {
            manager.add_bot(
                bot.username.clone(),
                bot.password.clone(),
                bot.code.clone(),
                bot.method.clone(),
            );
        }

        Self {
            current_menu: "bots".to_string(),
            search_query: "".to_string(),
            manager,
            new_bot_username: "".to_string(),
            new_bot_password: "".to_string(),
            new_bot_code: "".to_string(),
            new_bot_method: ELoginMethod::LEGACY,
            show_add_bot_dialog: false,
            bots: json.bots,
            selected_bot_name: "".to_string(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Mori");
                ui.separator();
                if ui.button("Bots").clicked() {
                    self.current_menu = "bots".to_string();
                }
                if ui.button("World").clicked() {
                    self.current_menu = "world".to_string();
                }
                if ui.button("Item database").clicked() {
                    self.current_menu = "item_database".to_string();
                }
                if ui.button("Features").clicked() {
                    self.current_menu = "features".to_string();
                }
                if ui.button("Settings").clicked() {
                    self.current_menu = "settings".to_string();
                }
                ui.separator();
                if ui.button("Add bot").clicked() {
                    self.show_add_bot_dialog = true;
                }
            });
            ui.separator();
            if self.current_menu == "bots" {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("bots_grid")
                            .min_col_width(100.0)
                            .max_col_width(100.0)
                            .show(ui, |ui| {
                                ui.label("Username");
                                ui.label("Password");
                                ui.label("2FA Code");
                                ui.label("Login Method");
                                ui.end_row();
                                for bot in &self.bots {
                                    if ui.button(bot.username.clone()).clicked() {
                                        self.selected_bot_name = bot.username.clone();
                                    }
                                    ui.label(bot.password.clone());
                                    ui.label(bot.code.clone());
                                    ui.label(format!("{:?}", bot.method));
                                    ui.end_row();
                                }
                            });
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("Bot info");
                        if !self.selected_bot_name.is_empty() {
                            if let Some(bot_mutex) = self.manager.get_bot(&self.selected_bot_name) {
                                let bot = bot_mutex.lock().unwrap();
                                ui.horizontal(|ui| {
                                    ui.label("Status:");
                                    ui.label(bot.info.status.clone());
                                    ui.separator();
                                    ui.label("Ping:");
                                    ui.label(bot.info.ping.clone().to_string());
                                });
                                ui.horizontal(|ui| {
                                    ui.label("World:");
                                    let world = if bot.world.name.is_empty() {
                                        "EXIT".to_string()
                                    } else {
                                        bot.world.name.clone()
                                    };
                                    ui.label(world);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Position:");
                                    ui.label(bot.position.x.to_string());
                                    ui.label(bot.position.y.to_string());
                                });
                            }
                        }
                    });
                });
            } else if self.current_menu == "item_database" {
                ui.horizontal(|ui| {
                    ui.label("Database version:");
                    ui.label(self.manager.items_database.version.to_string());
                    ui.separator();
                    ui.label("Items count:");
                    ui.label(self.manager.items_database.item_count.to_string());
                    ui.separator();
                    ui.text_edit_singleline::<String>(&mut self.search_query);
                });
                ui.separator();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    let text_style = egui::TextStyle::Body;
                    let row_height = ui.text_style_height(&text_style);
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        (self.manager.items_database.item_count) as usize,
                        |ui, row_range| {
                            ui.vertical(|ui| {
                                egui::Grid::new("item_database_grid")
                                    .min_col_width(300.0)
                                    .max_col_width(300.0)
                                    .show(ui, |ui| {
                                        for i in row_range {
                                            ui.label(
                                                self.manager.items_database.items[&(i as u32)]
                                                    .name
                                                    .clone(),
                                            );
                                            ui.end_row();
                                        }
                                    });
                            })
                        },
                    );
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("Item info");
                    });
                });
            } else {
                ui.label("Not implemented yet");
            }

            if self.show_add_bot_dialog {
                let mut close_dialog = false;
                egui::Window::new("Add bot")
                    .resizable(false)
                    .open(&mut self.show_add_bot_dialog)
                    .show(ctx, |ui| {
                        egui::Grid::new("add_bot_grid")
                            .min_col_width(100.0)
                            .max_col_width(100.0)
                            .show(ui, |ui| {
                                ui.label("Username");
                                ui.text_edit_singleline(&mut self.new_bot_username);
                                ui.end_row();
                                ui.label("Password");
                                ui.text_edit_singleline(&mut self.new_bot_password);
                                ui.end_row();
                                ui.label("2FA Code");
                                ui.text_edit_singleline(&mut self.new_bot_code);
                                ui.end_row();
                                ui.label("Login Method");
                                egui::ComboBox::from_id_source("login_method")
                                    .selected_text(format!("{:?}", self.new_bot_method))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.new_bot_method,
                                            ELoginMethod::LEGACY,
                                            "LEGACY",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_bot_method,
                                            ELoginMethod::GOOGLE,
                                            "GOOGLE",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_bot_method,
                                            ELoginMethod::APPLE,
                                            "APPLE",
                                        );
                                        ui.selectable_value(
                                            &mut self.new_bot_method,
                                            ELoginMethod::UBISOFT,
                                            "UBISOFT",
                                        );
                                    });
                                ui.end_row();
                            });
                        if ui.button("Add").clicked() {
                            self.manager.add_bot(
                                self.new_bot_username.clone(),
                                self.new_bot_password.clone(),
                                self.new_bot_code.clone(),
                                self.new_bot_method.clone(),
                            );
                            let mut data = serde_json::from_str::<Data>(
                                &fs::read_to_string("data.json").unwrap(),
                            )
                            .unwrap();
                            data.bots.push(Bot {
                                username: self.new_bot_username.clone(),
                                password: self.new_bot_password.clone(),
                                code: self.new_bot_code.clone(),
                                method: self.new_bot_method.clone(),
                            });
                            fs::write("data.json", &serde_json::to_string_pretty(&data).unwrap())
                                .unwrap();
                            self.new_bot_username.clear();
                            self.new_bot_password.clear();
                            self.new_bot_code.clear();
                            self.new_bot_method = ELoginMethod::LEGACY;
                            close_dialog = true;
                        }
                    });
                if close_dialog {
                    self.show_add_bot_dialog = false;
                }
            }
        });
    }
}
