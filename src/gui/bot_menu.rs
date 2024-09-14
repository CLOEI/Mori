use std::sync::{Arc, RwLock};
use std::thread;

use eframe::egui::{self, Ui};
use egui::include_image;
use crate::{bot::warp, manager::bot_manager::BotManager, types::config::BotConfig, utils, Bot};
use crate::bot::leave;
use crate::gui::inventory::Inventory;
use crate::gui::world_map::WorldMap;

#[derive(Default)]
pub struct BotMenu {
    pub selected_bot: String,
    pub warp_name: String,
    pub bots: Vec<BotConfig>,
    pub current_menu: String,
    pub world_map: WorldMap,
    pub inventory: Inventory,
}

impl BotMenu {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<BotManager>>) {
        self.bots = utils::config::get_bots();
        self.selected_bot = utils::config::get_selected_bot();
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("bots_grid")
                    .min_col_width(150.0)
                    .max_col_width(150.0)
                    .show(ui, |ui| {
                        for bot in self.bots.clone() {
                            let payload = utils::textparse::parse_and_store_as_vec(&bot.payload);
                            if ui.add_sized(ui.available_size(), egui::Button::new(payload[0].clone()).truncate()).clicked() {
                                self.selected_bot = payload[0].clone();
                                utils::config::set_selected_bot(self.selected_bot.clone());
                            }
                            ui.end_row();
                        }
                    })
            });
            ui.separator();
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                ui.vertical(|ui| {
                    if ui.add_sized([30.0, 30.0], egui::Button::image(
                        include_image!("../../assets/info.png"),
                    )).clicked() {
                        self.current_menu = "bot_info".to_string();
                    }
                    if ui.add_sized([30.0, 30.0], egui::Button::image(
                        include_image!("../../assets/earth.png"),
                    )).clicked() {
                        self.current_menu = "world_map".to_string();
                    }
                    if ui.add_sized([30.0, 30.0], egui::Button::image(
                        include_image!("../../assets/backpack.png"),
                    )).clicked() {
                        self.current_menu = "inventory".to_string();
                    }
                    if ui.add_sized([30.0, 30.0], egui::Button::image(
                        include_image!("../../assets/blocks.png"),
                    )).clicked() {
                        self.current_menu = "features".to_string();
                    }
                });

                let available_width = ui.available_width();
                // 9 is the pixel from the window border
                let half_width = (available_width - 9.0) / 2.0;
                if self.current_menu.is_empty() || self.current_menu == "bot_info" {
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
                                            let bot = {
                                                let manager = manager.read().unwrap();

                                                match manager.get_bot(&self.selected_bot) {
                                                    Some(bot) => Some(bot.clone()),
                                                    None => None,
                                                }
                                            };
                                            if let Some(bot) = bot {
                                                let (username, status, ping, world_name, timeout) = {
                                                    let info = bot.info.read().unwrap();
                                                    let world = bot.world.read().unwrap();
                                                    (
                                                        info.login_info.tank_id_name.clone(),
                                                        info.status.clone(),
                                                        info.ping.clone().to_string(),
                                                        world.name.clone(),
                                                        info.timeout.clone(),
                                                    )
                                                };
                                                ui.label("GrowID");
                                                ui.add(egui::Label::new(username).truncate());
                                                ui.end_row();
                                                ui.label("Status");
                                                ui.add(egui::Label::new(status).truncate());
                                                ui.end_row();
                                                ui.label("Ping");
                                                ui.label(ping);
                                                ui.end_row();
                                                ui.label("World");
                                                ui.label(world_name);
                                                ui.end_row();
                                                ui.label("Timeout");
                                                ui.label(timeout.to_string());
                                                ui.end_row();
                                            } else {
                                                ui.label("GrowID");
                                                ui.label("EMPTY");
                                                ui.end_row();
                                                ui.label("Status");
                                                ui.label("EMPTY");
                                                ui.end_row();
                                                ui.label("Ping");
                                                ui.label("EMPTY");
                                                ui.end_row();
                                                ui.label("World");
                                                ui.label("EXIT");
                                                ui.end_row();
                                                ui.label("Timeout");
                                                ui.label("0");
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
                                    if ui.button("Leave").clicked() {
                                        if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                            let bot_clone = bot.clone();
                                            thread::spawn(move || {
                                                leave(&bot_clone);
                                            });
                                        }
                                    }
                                    if ui.button("Warp").clicked() {
                                        if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                            let bot_clone = bot.clone();
                                            let world_name = self.warp_name.clone();
                                            thread::spawn(move || {
                                                warp(&bot_clone, world_name);
                                            });
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
                                            let bot = {
                                                let manager = manager.read().unwrap();

                                                match manager.get_bot(&self.selected_bot) {
                                                    Some(bot) => Some(bot.clone()),
                                                    None => None,
                                                }
                                            };
                                            if let Some(bot) = bot {
                                                let (ip, port) = {
                                                    let server = bot.server.read().unwrap();
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
                                            let bot = {
                                                let manager = manager.read().unwrap();

                                                match manager.get_bot(&self.selected_bot) {
                                                    Some(bot) => Some(bot.clone()),
                                                    None => None,
                                                }
                                            };
                                            if let Some(bot) = bot {
                                                let (payload, code, method) = {
                                                    let info = bot.info.read().unwrap();
                                                    (
                                                        info.payload.clone(),
                                                        info.recovery_code.clone(),
                                                        info.login_method.clone(),
                                                    )
                                                };
                                                ui.label("Username");
                                                ui.add(egui::Label::new(&payload[0]).truncate());
                                                ui.end_row();
                                                ui.label("Password");
                                                ui.label(&payload[1]);
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
                                            let bot = {
                                                let manager = manager.read().unwrap();

                                                match manager.get_bot(&self.selected_bot) {
                                                    Some(bot) => Some(bot.clone()),
                                                    None => None,
                                                }
                                            };
                                            if let Some(bot) = bot {
                                                let net_id = bot.state.read().unwrap().net_id.clone();
                                                let token = bot.info.read().unwrap().token.clone();
                                                let is_banned = bot.state.read().unwrap().is_banned.clone();
                                                let position = bot.position.read().unwrap().clone();
                                                ui.label("NetID");
                                                ui.label(net_id.to_string());
                                                ui.end_row();
                                                ui.label("Token");
                                                ui.add(egui::Label::new(token).truncate());
                                                ui.end_row();
                                                ui.label("Is Banned");
                                                ui.label(is_banned.to_string());
                                                ui.end_row();
                                                ui.label("Gems");
                                                ui.label(bot.state.read().unwrap().gems.to_string());
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
                } else if self.current_menu == "world_map" {
                    ui.allocate_ui(egui::vec2(available_width, ui.available_height()), |ui| {
                        self.world_map.render(ui, &manager);
                    });
                } else if self.current_menu == "inventory" {
                    ui.allocate_ui(egui::vec2(available_width, ui.available_height()), |ui| {
                        self.inventory.render(ui, &manager);
                    });
                } else if self.current_menu == "features" {
                    ui.allocate_ui(egui::vec2(available_width, ui.available_height()), |ui| {
                        ui.label("Not implemented yet");
                    });
                } else {
                    ui.label("How?");
                }
            });
        });
    }
}
