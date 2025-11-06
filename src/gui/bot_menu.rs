use std::sync::{Arc, RwLock};

use crate::gui::add_bot_dialog::AddBotDialog;
use crate::manager::bot_manager::BotManager;
use eframe::egui::{self, Ui};
use egui::scroll_area::ScrollBarVisibility;
use egui::{Color32, UiBuilder};

#[derive(Default)]
pub struct BotMenu {
    pub selected_bot: String,
    pub warp_name: String,
    pub current_menu: String,
}

impl BotMenu {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        add_bot_dialog: &mut AddBotDialog,
        manager: &Arc<RwLock<BotManager>>,
    ) {
        ui.add_space(4.0);
        ui.scope_builder(
            UiBuilder::new().layout(egui::Layout::left_to_right(egui::Align::Min)),
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
                                if ui
                                    .add(egui::Button::new(egui::RichText::new(format!(
                                        "{} Add bot",
                                        egui_material_icons::icons::ICON_ADD
                                    ))))
                                    .clicked()
                                {
                                    add_bot_dialog.open = true;
                                }
                                if ui
                                    .add(egui::Button::new(egui::RichText::new(format!(
                                        "{} Remove bot",
                                        egui_material_icons::icons::ICON_REMOVE
                                    ))))
                                    .clicked()
                                {
                                    let manager_clone = Arc::clone(manager);
                                    let selected_bot = self.selected_bot.clone();
                                    std::thread::spawn(move || {
                                        manager_clone.write().unwrap().remove_bot(&selected_bot);
                                    });
                                }
                            });
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("bot_list")
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        // Clone the bots list to release the manager lock before accessing bot mutexes
                                        let bots: Vec<_> = manager
                                            .read()
                                            .unwrap()
                                            .bots
                                            .iter()
                                            .map(|(bot, _)| bot.clone())
                                            .collect();

                                        if bots.is_empty() {
                                            ui.centered_and_justified(|ui| {
                                                ui.add_sized(
                                                    [ui.available_width(), 0.0],
                                                    egui::Label::new("No bots added"),
                                                );
                                            });
                                        } else {
                                            for bot in bots {
                                                let (label, selectable_username) =
                                                    match bot.auth.try_login_info() {
                                                        Some(guard) => guard
                                                            .as_ref()
                                                            .map(|info| {
                                                                let username =
                                                                    info.tank_id_name.clone();
                                                                (username.clone(), Some(username))
                                                            })
                                                            .unwrap_or_else(|| {
                                                                (
                                                                    "Preparing credentials..."
                                                                        .to_string(),
                                                                    None,
                                                                )
                                                            }),
                                                        None => (
                                                            "Connecting to server...".to_string(),
                                                            None,
                                                        ),
                                                    };

                                                let response = ui.add_sized(
                                                    [ui.available_width(), 0.0],
                                                    egui::Button::new(label.clone()).truncate(),
                                                );

                                                if let Some(username) = selectable_username {
                                                    if response.clicked() {
                                                        self.selected_bot = username;
                                                    }
                                                } else {
                                                    response
                                                        .on_hover_text("Bot is still connecting");
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
                        if ui
                            .add_sized(
                                [30.0, 30.0],
                                egui::Button::new(egui::RichText::new(
                                    egui_material_icons::icons::ICON_INFO,
                                )),
                            )
                            .on_hover_text("Bot Info")
                            .clicked()
                        {
                            self.current_menu = "bot_info".to_string();
                        }
                        if ui
                            .add_sized(
                                [30.0, 30.0],
                                egui::Button::new(egui::RichText::new(
                                    egui_material_icons::icons::ICON_TERMINAL,
                                )),
                            )
                            .on_hover_text("Terminal")
                            .clicked()
                        {
                            self.current_menu = "terminal".to_string();
                        }
                    });
                }
                if self.current_menu.is_empty() || self.current_menu == "bot_info" {
                    if let Some(bot) = bot {
                        ui.allocate_ui(
                            egui::vec2(ui.available_width() / 2.0, ui.available_height()),
                            |ui| {
                                ui.vertical(|ui| {
                                    ui.group(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label("Bot info");
                                            ui.separator();
                                            egui::Grid::new("bot_info")
                                                .min_col_width(120.0)
                                                .max_col_width(120.0)
                                                .show(ui, |ui| {
                                                    let username = bot
                                                        .auth
                                                        .try_login_info()
                                                        .and_then(|guard| {
                                                            guard.as_ref().map(|info| {
                                                                info.tank_id_name.clone()
                                                            })
                                                        })
                                                        .unwrap_or_else(|| {
                                                            "Loading GrowID...".to_string()
                                                        });
                                                    let enet_status = bot.enet_status();
                                                    let peer_status = bot.peer_status();
                                                    let ping = bot.runtime.ping();
                                                    let world_name = {
                                                        bot.world.data.lock().unwrap().name.clone()
                                                    };
                                                    let timeout = bot.timeout();

                                                    ui.label("GrowID");
                                                    ui.add(egui::Label::new(username).truncate());
                                                    ui.end_row();
                                                    ui.label("ENet Status");
                                                    ui.add(
                                                        egui::Label::new(format!(
                                                            "{:?}",
                                                            enet_status
                                                        ))
                                                        .truncate(),
                                                    );
                                                    ui.end_row();
                                                    ui.label("Peer Status");
                                                    ui.add(
                                                        egui::Label::new(format!(
                                                            "{:?}",
                                                            peer_status
                                                        ))
                                                        .truncate(),
                                                    );
                                                    ui.end_row();
                                                    ui.label("Ping");
                                                    ui.label(ping.to_string());
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
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Min),
                                            |ui| {
                                                if ui.button("Accept access").clicked() {
                                                    let bot_clone = bot.clone();
                                                    std::thread::spawn(move || {
                                                        bot_clone.accept_access();
                                                    });
                                                }
                                                if ui.button("Leave").clicked() {
                                                    let bot_clone = bot.clone();
                                                    std::thread::spawn(move || {
                                                        bot_clone.leave();
                                                    });
                                                }
                                                if ui.button("Warp").clicked() {
                                                    let bot_clone = bot.clone();
                                                    let world_name = self.warp_name.clone();
                                                    std::thread::spawn(move || {
                                                        bot_clone.warp(world_name);
                                                    });
                                                }
                                            },
                                        );
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
                                                    let (server, port) = {
                                                        bot.auth
                                                            .server_data()
                                                            .as_ref()
                                                            .map(|data| {
                                                                (data.server.clone(), data.port)
                                                            })
                                                            .unwrap_or_else(|| {
                                                                ("N/A".to_string(), 0)
                                                            })
                                                    };
                                                    ui.label("Server");
                                                    ui.label(server);
                                                    ui.end_row();
                                                    ui.label("Port");
                                                    ui.label(port.to_string());
                                                    ui.end_row();
                                                });
                                        });
                                        ui.add_space(ui.available_height());
                                    });
                                });
                            },
                        );
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Login info");
                                    ui.separator();
                                    egui::Grid::new("login_info")
                                        .min_col_width(120.0)
                                        .max_col_width(120.0)
                                        .show(ui, |ui| {
                                            let (username, password, method) = {
                                                let login_via = bot.auth.login_via();
                                                match login_via {
                                                    gt_core::types::bot::LoginVia::LEGACY(
                                                        creds,
                                                    ) => (
                                                        creds[0].clone(),
                                                        creds[1].clone(),
                                                        "LEGACY".to_string(),
                                                    ),
                                                    gt_core::types::bot::LoginVia::GOOGLE => (
                                                        "N/A".to_string(),
                                                        "N/A".to_string(),
                                                        "GOOGLE".to_string(),
                                                    ),
                                                    gt_core::types::bot::LoginVia::APPLE => (
                                                        "N/A".to_string(),
                                                        "N/A".to_string(),
                                                        "APPLE".to_string(),
                                                    ),
                                                    gt_core::types::bot::LoginVia::LTOKEN(_) => (
                                                        "N/A".to_string(),
                                                        "N/A".to_string(),
                                                        "LTOKEN".to_string(),
                                                    ),
                                                }
                                            };
                                            ui.label("Username");
                                            ui.add(egui::Label::new(username).truncate());
                                            ui.end_row();
                                            ui.label("Password");
                                            ui.label(password);
                                            ui.end_row();
                                            ui.label("Login Method");
                                            ui.label(method);
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
                                            let net_id = bot.runtime.net_id();
                                            let user_id = bot.runtime.user_id();
                                            let ltoken = bot
                                                .auth
                                                .try_login_info()
                                                .and_then(|guard| {
                                                    guard.as_ref().map(|info| info.ltoken.clone())
                                                })
                                                .unwrap_or_default();
                                            let gems = bot.inventory.gems();
                                            let position = bot.movement.position();

                                            ui.label("NetID");
                                            ui.label(net_id.to_string());
                                            ui.end_row();
                                            ui.label("UserID");
                                            ui.label(user_id.to_string());
                                            ui.end_row();
                                            ui.label("Token");
                                            ui.add(egui::Label::new(ltoken).truncate());
                                            ui.end_row();
                                            ui.label("Gems");
                                            ui.label(gems.to_string());
                                            ui.end_row();
                                            ui.label("Position");
                                            ui.horizontal(|ui| {
                                                ui.label((position.0 / 32.0).floor().to_string());
                                                ui.separator();
                                                ui.label((position.1 / 32.0).floor().to_string());
                                            });
                                            ui.end_row();
                                        });
                                });
                                ui.add_space(ui.available_height());
                            });
                        });
                    } else {
                        ui.with_layout(
                            egui::Layout::top_down_justified(egui::Align::Center),
                            |ui| {
                                ui.add_space(ui.available_height() / 2.0 - 25.0);
                                ui.vertical_centered(|ui| {
                                    ui.label("Select a bot to view info");
                                });
                                ui.add_space(ui.available_height() / 2.0 - 25.0);
                            },
                        );
                    }
                } else if self.current_menu == "terminal" {
                    if let Some(bot) = bot {
                        ui.allocate_ui(
                            egui::vec2(ui.available_width(), ui.available_height()),
                            |ui| {
                                egui::ScrollArea::vertical()
                                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                                    .auto_shrink(false)
                                    .stick_to_bottom(true)
                                    .show(ui, |ui| {
                                        let logs = bot.runtime.logs_snapshot();

                                        ui.vertical(|ui| {
                                            for log in logs.iter() {
                                                ui.label(log);
                                            }
                                            ui.add_space(10.0);
                                        });
                                    });
                            },
                        );
                    }
                } else {
                    ui.label("Invalid menu");
                }
            },
        );
    }
}
