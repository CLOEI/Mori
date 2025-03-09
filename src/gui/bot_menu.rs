use std::sync::{Arc, RwLock};
use std::thread;

use crate::gui::growscan::Growscan;
use crate::gui::inventory::Inventory;
use crate::core::features::auto_pnb::AutoPNB;
use crate::gui::scripting::Scripting;
use crate::gui::world_map::WorldMap;
use crate::texture_manager::TextureManager;
use crate::core::features::auto_spam::AutoSpam;
// use crate::gui::spam_feature_menu;
use crate::gui::players::PlayersScan;
use crate::{manager::bot_manager::BotManager, types::config::BotConfig, utils};
use eframe::egui::{self, Ui};
use egui::scroll_area::ScrollBarVisibility;
use egui::{Color32, UiBuilder};
use crate::core::features;
use crate::gui::add_bot_dialog::AddBotDialog;

#[derive(Default)]
pub struct BotMenu {
    pub selected_bot: String,
    pub warp_name: String,
    pub bots: Vec<BotConfig>,
    pub current_menu: String,
    pub current_feature: String,
    pub players_in_world: PlayersScan,
    pub auto_farm: AutoPNB,
    pub auto_spam: AutoSpam,
    pub world_map: WorldMap,
    pub inventory: Inventory,
    pub growscan: Growscan,
    pub scripting: Scripting,
    pub selected_feature: String,
}

impl BotMenu {
    pub fn render(&mut self, ui: &mut Ui, add_bot_dialog: &mut AddBotDialog, manager: &Arc<RwLock<BotManager>>, texture_manager: &Arc<RwLock<TextureManager>>) {
        self.bots = utils::config::get_bots();
        self.selected_bot = utils::config::get_selected_bot();
        ui.add_space(4.0);
        ui.allocate_new_ui(
            UiBuilder::new()
                .layout(egui::Layout::left_to_right(egui::Align::Min)),
            |ui| {
                let bot = {
                    let manager = manager.read().unwrap();

                    match manager.get_bot(&self.selected_bot) {
                        Some(bot) => Some(bot.clone()),
                        None => None,
                    }
                };
                ui.allocate_ui(
                    egui::vec2(ui.available_width() * 0.18, ui.available_height()),
                    |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.add(
                                    egui::Button::new(
                                        egui_phosphor::variants::fill::PLUS.to_owned() + " Add bot"
                                    )
                                ).clicked(){
                                    add_bot_dialog.open = true;
                                }
                                if ui.add(
                                    egui::Button::new(
                                        egui_phosphor::variants::fill::MINUS.to_owned() + "  Remove bot"
                                    )
                                ).clicked(){
                                    let selected_bot = utils::config::get_selected_bot();
                                    manager.write().unwrap().remove_bot(&selected_bot);
                                }
                            });
                            ui.separator();
                            egui::ScrollArea::vertical().id_salt("bot_list").show(ui, |ui| {
                                ui.vertical(|ui| {
                                    let bots_clone = self.bots.clone();
                                    if bots_clone.is_empty() {
                                        ui.centered_and_justified(|ui| {
                                            ui.add_sized([ui.available_width(), 0.0], egui::Label::new("No bots added"));
                                        });
                                    } else {
                                        for bot in self.bots.clone() {
                                            let payload = utils::textparse::parse_and_store_as_vec(&bot.payload);
                                            if ui.add_sized([ui.available_width(), 0.0], egui::Button::new(payload[0].clone()).truncate()).clicked() {
                                                self.selected_bot = payload[0].clone();
                                                utils::config::set_selected_bot(self.selected_bot.clone());
                                            }
                                        }
                                    }
                                });
                            });
                        });
                    },
                );
                ui.separator();
                if let Some(_) = bot {
                    ui.vertical(|ui| {
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::INFO),
                        )).clicked() {
                            self.current_menu = "bot_info".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::PLANET)
                        )).clicked() {
                            self.current_menu = "world_map".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::BACKPACK),
                        )).clicked() {
                            self.current_menu = "inventory".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::TARGET),
                        )).clicked() {
                            self.current_menu = "radar".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::USER_LIST),
                        )).clicked() {
                            self.current_menu = "players_in_world".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::DIAMONDS_FOUR),
                        )).clicked() {
                            self.current_menu = "features".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::CODE),
                        )).clicked() {
                            self.current_menu = "scripting".to_string();
                        }
                        if ui.add_sized([30.0, 30.0], egui::Button::new(
                            egui::RichText::new(egui_phosphor::variants::fill::TERMINAL_WINDOW),
                        )).clicked() {
                            self.current_menu = "terminal".to_string();
                        }
                    });
                }
                if self.current_menu.is_empty() || self.current_menu == "bot_info" {
                    if let Some(bot) = bot {
                        ui.allocate_ui(egui::vec2(ui.available_width() / 2.0, ui.available_height()), |ui| {
                            ui.vertical(|ui| {
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("Bot info");
                                        ui.separator();
                                        egui::Grid::new("bot_info")
                                            .min_col_width(120.0)
                                            .max_col_width(120.0)
                                            .show(ui, |ui| {
                                                let (username, status, ping, world_name, timeout) = {
                                                    let info = bot.info.lock().unwrap();
                                                    let temp: std::sync::RwLockReadGuard<'_, crate::types::bot_info::TemporaryData> = bot.temporary_data.read().unwrap();
                                                    let world = bot.world.read().unwrap();
                                                    (
                                                        info.login_info.tank_id_name.clone(),
                                                        info.status.clone(),
                                                        temp.ping.clone().to_string(),
                                                        world.name.clone(),
                                                        temp.timeout.clone(),
                                                    )
                                                };
                                                ui.label("GrowID");
                                                ui.add(egui::Label::new(username).truncate());
                                                ui.end_row();
                                                ui.label("Status");
                                                ui.add(egui::Label::new(status.to_string()).truncate());
                                                ui.end_row();
                                                ui.horizontal_centered(|ui| {
                                                    if ui.button("Connect").clicked() {
                                                        if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                                            let bot_clone = bot.clone();
                                                            thread::spawn(move || {
                                                                bot_clone.connect_bot();
                                                            });
                                                        }
                                                    }
                                                    if ui.button("Disconnect").clicked() {
                                                        if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                                            let bot_clone = bot.clone();
                                                            thread::spawn(move || {
                                                                bot_clone.disconnect_bot();
                                                            });
                                                        }
                                                    }
                                                    if ui.button("Reconnect").clicked() {
                                                        if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                                            let bot_clone = bot.clone();
                                                            thread::spawn(move || {
                                                                bot_clone.reconnect_bot();
                                                            });
                                                        }
                                                    }
                                                });
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
                                            });
                                    });
                                });
                                ui.allocate_space(egui::vec2(ui.available_width(), 5.0));
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
                                        if ui.button("Accept access").clicked() {
                                            if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                                let bot_clone = bot.clone();
                                                thread::spawn(move || {
                                                    bot_clone.accept_access();
                                                });
                                            }
                                        }
                                        // if ui.button("Relog").clicked() {
                                        //     if let Some(bot) = manager.read().unwrap().get_bot(&self.selected_bot) {
                                        //         let bot_clone = bot.clone();
                                        //         thread::spawn(move || {
                                        //             bot_clone.relog();
                                        //         });
                                        //     }
                                        // }
                                        if ui.button("Leave").clicked() {
                                            let bot_clone = bot.clone();
                                            thread::spawn(move || {
                                                bot_clone.leave();
                                            });
                                        }
                                        if ui.button("Warp").clicked() {
                                            let bot_clone = bot.clone();
                                            let world_name = self.warp_name.clone();
                                            thread::spawn(move || {
                                                bot_clone.warp(world_name);
                                            });
                                        }
                                    });
                                });
                                ui.allocate_space(egui::vec2(ui.available_width(), 5.0));
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("Server");
                                        ui.separator();
                                        egui::Grid::new("bot_server")
                                            .min_col_width(120.0)
                                            .max_col_width(120.0)
                                            .show(ui, |ui| {
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
                                            });
                                    });
                                    ui.add_space(ui.available_height());
                                });
                            });
                        });
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Login info");
                                    ui.separator();
                                    egui::Grid::new("login_info")
                                        .min_col_width(120.0)
                                        .max_col_width(120.0)
                                        .show(ui, |ui| {
                                            let (payload, code, method) = {
                                                let info = bot.info.lock().unwrap();
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
                                        });
                                });
                            });
                            ui.allocate_space(egui::vec2(ui.available_width(), 5.0));
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("State");
                                    ui.separator();
                                    egui::Grid::new("bot_state")
                                        .min_col_width(120.0)
                                        .max_col_width(120.0)
                                        .show(ui, |ui| {
                                            let net_id = bot.state.lock().unwrap().net_id.clone();
                                            let token = bot.info.lock().unwrap().token.clone();
                                            let is_banned = bot.state.lock().unwrap().is_banned.clone();
                                            let position = bot.position.lock().unwrap().clone();
                                            ui.label("NetID");
                                            ui.label(net_id.to_string());
                                            ui.end_row();
                                            ui.label("Token");
                                            ui.add(egui::Label::new(token).truncate());
                                            ui.end_row();
                                            ui.label("Account age");
                                            let (install_date, playtime) = {
                                                let bot = bot.state.lock().unwrap();
                                                (bot.install_date, bot.playtime)
                                            };
                                            if install_date > 0 {
                                                ui.add(egui::Label::new(self.format_duration_from_epoch(install_date)).truncate());
                                            } else {
                                                ui.add(egui::Label::new("0").truncate());
                                            }
                                            ui.end_row();
                                            ui.label("Account playtime");
                                            if playtime > 0 {
                                                ui.add(egui::Label::new(self.format_duration(playtime)).truncate());
                                            } else {
                                                ui.add(egui::Label::new("0").truncate());
                                            }
                                            ui.end_row();
                                            ui.label("Is Banned");
                                            ui.label(is_banned.to_string());
                                            ui.end_row();
                                            ui.label("Level");
                                            ui.label(bot.state.lock().unwrap().level.to_string());
                                            ui.end_row();
                                            ui.label("Gems");
                                            ui.label(bot.state.lock().unwrap().gems.to_string());
                                            ui.end_row();
                                            ui.label("Position");
                                            ui.horizontal(|ui| {
                                                ui.label((position.x / 32.0).floor().to_string());
                                                ui.separator();
                                                ui.label((position.y / 32.0).floor().to_string());
                                            });
                                            ui.end_row();
                                        });
                                });
                                ui.add_space(ui.available_height());
                            });
                        });
                    } else {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            ui.add_space(ui.available_height() / 2.0 - 25.0);
                            ui.vertical_centered(|ui| {
                                ui.add(egui::Label::new(egui::RichText::new(egui_phosphor::variants::fill::USER).size(50.0)));
                                ui.label("Select a bot to view info");
                            });
                            ui.add_space(ui.available_height() / 2.0 - 25.0);
                        });
                    }
                } else if self.current_menu == "world_map" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        self.world_map.render(ui, &manager, &texture_manager);
                    });
                } else if self.current_menu == "inventory" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        self.inventory.render(ui, &manager);
                    });
                } else if self.current_menu == "players_in_world" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        self.players_in_world.render(ui, &manager);
                    });
                } else if self.current_menu == "radar" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        self.growscan.render(ui, &manager);
                    });
                } else if self.current_menu == "features" {
                    ui.vertical(|ui| {
                        ui.heading("Features");
                        ui.horizontal(|ui| {
                            if ui.button("PNB").clicked() {
                                self.current_feature = "pnb".to_string();
                            }
                            if ui.button("Spam").clicked() {
                                self.current_feature = "spam".to_string();
                            }
                            if ui.button("Auto Provider").clicked() {
                                self.selected_feature = "auto_provider".to_string();
                                let bot = {
                                    let manager = manager.read().unwrap();
                                    manager.get_bot(&self.selected_bot).unwrap().clone()
                                };
                                features::auto_provider::start(&bot, 928);
                            }
                        });
                        ui.separator();
                        match self.current_feature.as_str() {
                            "pnb" => {
                                ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                                    self.auto_farm.render(ui, manager.clone());
                                });
                            }
                            "spam" => {
                                ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                                    self.auto_spam.render(ui, manager.clone());
                                });
                            }
                            "auto_provider" => {
                                ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                                    self.auto_spam.render(ui, manager.clone());
                                });
                            }
                            _ => {
                                ui.centered_and_justified(|ui| {
                                    ui.label("Please select a valid feature.");
                                });
                            }
                        }
                    });
                } else if self.current_menu == "scripting" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        self.scripting.render(ui, &manager);
                    });
                } else if self.current_menu == "terminal" {
                    ui.allocate_ui(egui::vec2(ui.available_width(), ui.available_height()), |ui| {
                        egui::ScrollArea::vertical()
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                            .auto_shrink(false)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                let logs = {
                                    let manager = manager.read().unwrap();

                                    match manager.get_bot(&self.selected_bot) {
                                        Some(bot) => {
                                            let logs = bot.logs.lock().unwrap();
                                            Some(logs.clone())
                                        }
                                        None => None,
                                    }
                                };

                                ui.vertical(|ui| {
                                    if let Some(logs) = logs {
                                        for log in logs.iter() {
                                            let data = log.split("|").collect::<Vec<&str>>();
                                            ui.horizontal(|ui| {
                                                match data[0] {
                                                    "info" => {
                                                        ui.label(egui::RichText::new(egui_phosphor::variants::fill::INFO).color(Color32::from_rgb(0, 123, 255)).size(16.0));
                                                        ui.add(egui::Label::new(data[1]).wrap());
                                                    }
                                                    "warn" => {
                                                        ui.label(egui::RichText::new(egui_phosphor::variants::fill::WARNING).color(Color32::from_rgb(255, 193, 7)).size(16.0));
                                                        ui.add(egui::Label::new(data[1]).wrap());
                                                    }
                                                    "error" => {
                                                        ui.label(egui::RichText::new(egui_phosphor::variants::fill::BUG).color(Color32::from_rgb(220, 53, 69)).size(16.0));
                                                        ui.add(egui::Label::new(data[1]).wrap());
                                                    }
                                                    _ => {
                                                        ui.label(log);
                                                    }
                                                };
                                            });
                                        }
                                    }
                                    ui.add_space(10.0);
                                });
                            });
                    });
                } else {
                    ui.label("How?");
                }
            },
        );
    }

    fn format_duration_from_epoch(&self, seconds: u32) -> String {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u32;
        let elapsed = current_time - seconds;
        let years = elapsed / 31_536_000;
        let months = (elapsed % 31_536_000) / 2_592_000;
        let days = (elapsed % 2_592_000) / 86_400;
        format!("{} years, {} months, {} days", years, months, days)
    }

    fn format_duration(&self, seconds: u32) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let seconds = seconds % 60;
        format!("{:02} hours, {:02} minutes, {:02} seconds", hours, minutes, seconds)
    }
}
