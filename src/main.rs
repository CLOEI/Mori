mod bot;
mod gui;
mod manager;
mod types;
mod utils;

use std::fs;

use eframe::egui::{self, ViewportBuilder};
use gui::{add_bot_dialog::AddBotDialog, item_database::ItemDatabase, warp_dialog::WarpDialog};
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
    item_database: ItemDatabase,
    manager: Manager,
    add_bot_dialog: AddBotDialog,
    warp_dialog: WarpDialog,
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
            item_database: Default::default(),
            manager,
            add_bot_dialog: Default::default(),
            warp_dialog: Default::default(),
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
                    self.add_bot_dialog.open = true;
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
                                if ui.button("Warp").clicked() {
                                    self.warp_dialog.open = true;
                                }
                            }
                        }
                    });
                });
            } else if self.current_menu == "item_database" {
                self.item_database.render(ui, &mut self.manager, ctx);
            } else {
                ui.label("Not implemented yet");
            }
        });
        self.add_bot_dialog.render(&mut self.manager, ctx);
        self.warp_dialog
            .render(&self.selected_bot_name, &mut self.manager, ctx);
    }
}
