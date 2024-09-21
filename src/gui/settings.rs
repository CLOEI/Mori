use eframe::egui::{self, Ui};
use egui::include_image;
use crate::utils::config;

#[derive(Default)]
pub struct Settings {
    pub use_alternate: bool,
    pub timeout_delay: u32,
    pub findpath_delay: u32,
}

impl Settings {
    pub fn render(&mut self, ui: &mut Ui, _ctx: &egui::Context) {
        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([0.0, 20.0])
            .min_col_width(ui.available_width() / 2.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if ui.checkbox(&mut self.use_alternate, "Use alternate server").changed() {
                        config::set_use_alternate_server(self.use_alternate);
                    }
                    ui.add_space(10.0);
                    if ui.add(egui::Slider::new(&mut self.timeout_delay, 0..=60).integer().suffix("sec").text("Timeout delay")).changed() {
                        config::set_timeout(self.timeout_delay);
                    }
                    ui.add_space(10.0);
                    if ui.add(egui::Slider::new(&mut self.findpath_delay, 0..=1000).integer().suffix("sec").text("Findpath delay")).changed() {
                        config::set_findpath_delay(self.findpath_delay);
                    }
                });
                ui.vertical(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.hyperlink_to("CLOEI", "https://github.com/CLOEI");
                        ui.label("Created by:");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label("A passionate software developer");;
                    });
                    ui.add_space(20.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label("Contributors:");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.hyperlink_to("Foite", "https://github.com/foite");
                    });
                });
                ui.end_row();
            });
    }
}

