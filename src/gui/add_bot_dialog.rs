use crate::manager::bot_manager::{BotConfig, BotManager};
use eframe::egui::{self};
use gt_core::{Socks5Config, types::bot::LoginVia};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AddBotDialog {
    pub username: String,
    pub password: String,
    pub ltoken: String,
    pub method: LoginVia,
    pub use_proxy: bool,
    pub proxy_addr: String,
    pub proxy_username: String,
    pub proxy_password: String,
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
                            // Show different fields based on login method
                            match &self.method {
                                LoginVia::LEGACY(_) => {
                                    ui.label("Username");
                                    ui.text_edit_singleline(&mut self.username);
                                    ui.end_row();
                                    ui.label("Password");
                                    ui.text_edit_singleline(&mut self.password);
                                    ui.end_row();
                                }
                                LoginVia::LTOKEN(_) => {
                                    ui.label("LTOKEN");
                                    ui.text_edit_singleline(&mut self.ltoken);
                                    ui.end_row();
                                }
                                LoginVia::GOOGLE | LoginVia::APPLE => {
                                    ui.label("Note: Token will be fetched automatically");
                                    ui.end_row();
                                }
                            }

                            ui.label("Login Method");
                            egui::ComboBox::from_id_salt("login_method")
                                .selected_text(format!("{:?}", self.method))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.method,
                                        LoginVia::LEGACY([String::new(), String::new()]),
                                        "LEGACY",
                                    );
                                    ui.selectable_value(
                                        &mut self.method,
                                        LoginVia::GOOGLE,
                                        "GOOGLE",
                                    );
                                    ui.selectable_value(&mut self.method, LoginVia::APPLE, "APPLE");
                                    ui.selectable_value(
                                        &mut self.method,
                                        LoginVia::LTOKEN([
                                            String::new(),
                                            String::new(),
                                            String::new(),
                                            String::new(),
                                        ]),
                                        "LTOKEN",
                                    );
                                });
                            ui.end_row();

                            ui.checkbox(&mut self.use_proxy, "Use proxy");
                            ui.end_row();

                            // Show proxy fields if proxy is enabled
                            if self.use_proxy {
                                ui.label("Proxy Address");
                                ui.text_edit_singleline(&mut self.proxy_addr);
                                ui.end_row();
                                ui.label("Proxy Username");
                                ui.text_edit_singleline(&mut self.proxy_username);
                                ui.end_row();
                                ui.label("Proxy Password");
                                ui.text_edit_singleline(&mut self.proxy_password);
                                ui.end_row();
                            }
                        });
                    if ui.button("Add").clicked() {
                        let login_method = match &self.method {
                            LoginVia::LEGACY(_) => {
                                LoginVia::LEGACY([self.username.clone(), self.password.clone()])
                            }
                            LoginVia::LTOKEN(_) => {
                                let parts: Vec<&str> = self.ltoken.split(':').collect();
                                if parts.len() == 4 {
                                    LoginVia::LTOKEN([
                                        parts[0].to_string(),
                                        parts[1].to_string(),
                                        parts[2].to_string(),
                                        parts[3].to_string(),
                                    ])
                                } else {
                                    // Show error or return
                                    return;
                                }
                            }
                            LoginVia::GOOGLE => LoginVia::GOOGLE,
                            LoginVia::APPLE => LoginVia::APPLE,
                        };

                        let proxy = if self.use_proxy {
                            // Parse proxy address (format: "ip:port")
                            if let Ok(proxy_addr) = self.proxy_addr.parse::<SocketAddr>() {
                                Some(Socks5Config {
                                    proxy_addr,
                                    username: if self.proxy_username.is_empty() {
                                        None
                                    } else {
                                        Some(self.proxy_username.clone())
                                    },
                                    password: if self.proxy_password.is_empty() {
                                        None
                                    } else {
                                        Some(self.proxy_password.clone())
                                    },
                                })
                            } else {
                                // Invalid proxy address format
                                return;
                            }
                        } else {
                            None
                        };

                        let config = BotConfig {
                            login_method,
                            proxy,
                        };

                        BotManager::add_bot(Arc::clone(manager), config);

                        self.username.clear();
                        self.password.clear();
                        self.ltoken.clear();
                        self.proxy_addr.clear();
                        self.proxy_username.clear();
                        self.proxy_password.clear();
                        self.method = LoginVia::LEGACY([String::new(), String::new()]);
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.open = false;
            }
        }
    }
}
