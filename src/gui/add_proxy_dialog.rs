use crate::manager::proxy_manager::ProxyManager;
use crate::types::config;
use crate::utils;
use eframe::egui::{self};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct AddProxyDialog {
    pub payload: String,
    pub open: bool,
}

impl AddProxyDialog {
    pub fn render(&mut self, manager: &Arc<RwLock<ProxyManager>>, ctx: &egui::Context) {
        if self.open {
            let mut close_dialog = false;
            egui::Window::new("Add proxy")
                .resizable(false)
                .open(&mut self.open)
                .show(ctx, |ui| {
                    egui::Grid::new("add_bot_grid")
                        .min_col_width(100.0)
                        .max_col_width(100.0)
                        .show(ui, |ui| {
                            ui.label("Payload");
                            ui.text_edit_singleline(&mut self.payload);
                            ui.end_row();
                        });
                    if ui.button("Add").clicked() {
                        // TODO: Add validation
                        let payload = utils::textparse::parse_and_store_as_vec(&self.payload);
                        let proxy_config = config::Proxy {
                            ip: payload[0].clone(),
                            port: payload[1].parse::<u16>().unwrap(),
                            username: payload[2].clone(),
                            password: payload[3].clone(),
                        };
                        manager.write().unwrap().add(proxy_config);
                        self.payload.clear();
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.open = false;
            }
        }
    }
}
