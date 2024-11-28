use crate::gui::add_proxy_dialog::AddProxyDialog;
use crate::manager::proxy_manager::ProxyManager;
use eframe::egui::{self, Ui};
use paris::info;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct ProxyList {}

impl ProxyList {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        proxy_manager: &Arc<RwLock<ProxyManager>>,
        add_proxy_dialog: &mut AddProxyDialog,
        _ctx: &egui::Context,
    ) {
        ui.vertical(|ui| {
            if ui.button("Add proxy").clicked() {
                add_proxy_dialog.open = true;
            }
            ui.separator();
            egui::Grid::new("proxy_list_grid")
                .min_col_width(ui.available_width() / 5.0)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("IP");
                    ui.label("Port");
                    ui.label("Username");
                    ui.label("Password");
                    ui.label("Status");
                    ui.end_row();
                    let proxies = {
                        let proxy_manager = proxy_manager.read().unwrap();
                        proxy_manager.proxies.clone()
                    };
                    for (index, proxy_data) in proxies.iter().enumerate() {
                        let proxy = &proxy_data.proxy;
                        ui.label(proxy.ip.to_string());
                        ui.label(proxy.port.to_string());
                        ui.label(proxy.username.to_string());
                        ui.label(proxy.password.to_string());
                        let response = ui
                            .group(|ui| {
                                ui.label(proxy_data.status.to_string());
                            })
                            .response;
                        ui.end_row();

                        response.context_menu(|ui| {
                            if ui.button("Test").clicked() {
                                proxy_manager.write().unwrap().test(index);
                                ui.close_menu();
                            }
                            if ui.button("Remove").clicked() {
                                proxy_manager.write().unwrap().remove(index);
                                ui.close_menu();
                            }
                        });
                    }
                });
        });
    }
}
