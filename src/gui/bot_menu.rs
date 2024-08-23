use std::thread;

use eframe::egui::{self, Ui};

use crate::{bot::warp, manager::Manager, types::config::BotConfig, Bot};

#[derive(Default)]
pub struct BotMenu {
    pub selected_bot: String,
    pub warp_name: String,
}

impl BotMenu {
    pub fn render(&mut self, ui: &mut Ui, bots: &Vec<BotConfig>, manager: &Manager) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("bots_grid")
                    .min_col_width(150.0)
                    .max_col_width(150.0)
                    .show(ui, |ui| {
                        ui.label("Bots");
                        ui.end_row();
                        for bot in bots {
                            if ui
                                .add(egui::Button::new(bot.username.clone()).truncate())
                                .clicked()
                            {
                                self.selected_bot = bot.username.clone();
                            }
                            ui.end_row();
                        }
                    })
            });
            ui.separator();
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let available_width = ui.available_width();
                // 9 is the pixel from the window border
                let half_width = (available_width - 9.0) / 2.0;
                ui.allocate_ui(egui::vec2(half_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Bot info");
                                ui.separator();
                                egui::Grid::new("bot_info")
                                    .min_col_width(120.0)
                                    .max_col_width(120.0)
                                    .show(ui, |ui| {
                                        if let Some(bot) = manager.get_bot(&self.selected_bot) {
                                            let (status, ping, world_name) = {
                                                let info = bot.info.lock().unwrap();
                                                let world = bot.world.lock().unwrap();
                                                (
                                                    info.status.clone(),
                                                    info.ping.clone().to_string(),
                                                    world.name.clone(),
                                                )
                                            };
                                            ui.label("Status");
                                            ui.add(egui::Label::new(status).truncate());
                                            ui.end_row();
                                            ui.label("Ping");
                                            ui.label(ping);
                                            ui.end_row();
                                            ui.label("World");
                                            ui.label(world_name);
                                            ui.end_row();
                                        } else {
                                            ui.label("Status");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("Ping");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("World");
                                            ui.label("EXIT");
                                            ui.end_row();
                                        }
                                    });
                            });
                        });
                        ui.allocate_space(egui::vec2(half_width, 5.0));
                        ui.group(|ui| {
                            ui.label("Warp");
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("World name");
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::singleline(&mut self.warp_name),
                                );
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if ui.button("Warp").clicked() {
                                    if let Some(bot) = manager.get_bot(&self.selected_bot) {
                                        let world_name = self.warp_name.clone();
                                        warp(bot, world_name);
                                    }
                                }
                            });
                        });
                        ui.allocate_space(egui::vec2(half_width, 5.0));
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Server");
                                ui.separator();
                                egui::Grid::new("bot_server")
                                    .min_col_width(120.0)
                                    .max_col_width(120.0)
                                    .show(ui, |ui| {
                                        if let Some(bot) = manager.get_bot(&self.selected_bot) {
                                            let (ip, port) = {
                                                let server = bot.server.lock().unwrap();
                                                (server.ip.clone(), server.port.clone().to_string())
                                            };
                                            ui.label("IP");
                                            ui.label(ip);
                                            ui.end_row();
                                            ui.label("Port");
                                            ui.label(port);
                                            ui.end_row();
                                        } else {
                                            ui.label("IP");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("Port");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                        }
                                    });
                            });
                            ui.add_space(ui.available_height());
                        });
                    });
                });

                ui.allocate_ui(egui::vec2(half_width, ui.available_height()), |ui| {
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Login info");
                                ui.separator();
                                egui::Grid::new("login_info")
                                    .min_col_width(120.0)
                                    .max_col_width(120.0)
                                    .show(ui, |ui| {
                                        if let Some(bot) = manager.get_bot(&self.selected_bot) {
                                            let (username, password, code, method) = {
                                                let info = bot.info.lock().unwrap();
                                                (
                                                    info.login_info.tank_id_name.clone(),
                                                    info.password.clone(),
                                                    info.recovery_code.clone(),
                                                    info.login_method.clone(),
                                                )
                                            };
                                            ui.label("Username");
                                            ui.add(egui::Label::new(username).truncate());
                                            ui.end_row();
                                            ui.label("Password");
                                            ui.label(password);
                                            ui.end_row();
                                            ui.label("2FA Code");
                                            ui.label(code);
                                            ui.end_row();
                                            ui.label("Login Method");
                                            ui.label(format!("{:?}", method));
                                            ui.end_row();
                                        } else {
                                            ui.label("Username");
                                            ui.add(egui::Label::new("EMPTY").truncate());
                                            ui.end_row();
                                            ui.label("Password");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("2FA Code");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("Login Method");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                        };
                                    });
                            });
                        });
                        ui.allocate_space(egui::vec2(half_width, 5.0));
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("State");
                                ui.separator();
                                egui::Grid::new("bot_state")
                                    .min_col_width(120.0)
                                    .max_col_width(120.0)
                                    .show(ui, |ui| {
                                        if let Some(bot) = manager.get_bot(&self.selected_bot) {
                                            let net_id = bot.state.lock().unwrap().net_id.clone();
                                            let token = bot.info.lock().unwrap().token.clone();
                                            let is_banned =
                                                bot.state.lock().unwrap().is_banned.clone();
                                            let position = bot.position.lock().unwrap().clone();
                                            ui.label("NetID");
                                            ui.label(net_id.to_string());
                                            ui.end_row();
                                            ui.label("Token");
                                            ui.add(egui::Label::new(token).truncate());
                                            ui.end_row();
                                            ui.label("Is Banned");
                                            ui.label(is_banned.to_string());
                                            ui.end_row();
                                            ui.label("Position");
                                            ui.horizontal(|ui| {
                                                ui.label((position.x / 32.0).to_string());
                                                ui.separator();
                                                ui.label((position.y / 32.0).to_string());
                                            });
                                            ui.end_row();
                                        } else {
                                            ui.label("NetID");
                                            ui.label("EMPTY");
                                            ui.end_row();
                                            ui.label("Token");
                                            ui.add(egui::Label::new("EMPTY").truncate());
                                            ui.end_row();
                                            ui.label("Is Banned");
                                            ui.label("False");
                                            ui.end_row();
                                            ui.label("Position");
                                            ui.horizontal(|ui| {
                                                ui.label("0");
                                                ui.separator();
                                                ui.label("0");
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });
                            ui.add_space(ui.available_height());
                        });
                    });
                });
            });
        });
    }
}
