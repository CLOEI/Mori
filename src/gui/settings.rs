use eframe::egui::{self, Ui};
use crate::utils::config;

#[derive(Default)]
pub struct Settings {
    pub use_alternate: bool,
}

impl Settings {
    pub fn render(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        ui.vertical(|ui| {
            if ui.checkbox(&mut self.use_alternate, "Use alternate server").changed() {
                config::set_use_alternate_server(self.use_alternate);
            }
        });
    }
}

