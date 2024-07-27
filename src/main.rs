mod bot;
mod manager;
mod types;
mod utils;

use eframe::egui::{self, ViewportBuilder};
use manager::Manager;
use types::e_login_method::ELoginMethod;

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
    item_search: String,
    manager: Manager,
    new_bot_username: String,
    new_bot_password: String,
    new_bot_code: String,
    new_bot_method: ELoginMethod,
    show_add_bot_dialog: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let manager = Manager::new().unwrap();

        Self {
            current_menu: "bots".to_string(),
            item_search: "".to_string(),
            manager,
            new_bot_username: "".to_string(),
            new_bot_password: "".to_string(),
            new_bot_code: "".to_string(),
            new_bot_method: ELoginMethod::LEGACY,
            show_add_bot_dialog: false,
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
                            });
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("Bot info");
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
                    ui.text_edit_singleline::<String>(&mut self.item_search);
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
