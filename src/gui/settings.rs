use crate::{
    utils::{captcha::CaptchaProvider, config},
};
use eframe::egui::{self, Ui};

#[derive(Default)]
pub struct Settings {
    pub use_alternate: bool,
    pub auto_collect: bool,
    pub timeout_delay: u32,
    pub findpath_delay: u32,
    pub captcha_provider: CaptchaProvider,
    pub captcha_api_key: String,
}

impl Settings {
    pub fn render(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([0.0, 20.0])
            .min_col_width(ui.available_width() / 2.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if ui
                        .checkbox(&mut self.use_alternate, "Use alternate server")
                        .changed()
                    {
                        config::set_use_alternate_server(self.use_alternate);
                    }
                    if ui
                        .checkbox(&mut self.auto_collect, "Use auto collect")
                        .changed()
                    {
                        config::set_auto_collect(self.auto_collect);
                    }
                    ui.add_space(10.0);
                    if ui
                        .add(
                            egui::Slider::new(&mut self.timeout_delay, 0..=60)
                                .integer()
                                .suffix("sec")
                                .text("Timeout delay"),
                        )
                        .changed()
                    {
                        config::set_timeout(self.timeout_delay);
                    }
                    ui.add_space(10.0);
                    if ui
                        .add(
                            egui::Slider::new(&mut self.findpath_delay, 0..=1000)
                                .integer()
                                .suffix("sec")
                                .text("Findpath delay"),
                        )
                        .changed()
                    {
                        config::set_findpath_delay(self.findpath_delay);
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Captcha provider:");
                        if ui
                            .selectable_value(
                                &mut self.captcha_provider,
                                CaptchaProvider::TwoCaptcha,
                                "2Captcha",
                            )
                            .changed()
                        {
                            config::set_captcha_provider(self.captcha_provider.clone());
                        };
                        if ui
                            .selectable_value(
                                &mut self.captcha_provider,
                                CaptchaProvider::CapSolver,
                                "CapSolver",
                            )
                            .changed()
                        {
                            config::set_captcha_provider(self.captcha_provider.clone());
                        };
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Captcha API key:");
                        ui.text_edit_singleline(&mut self.captcha_api_key);
                    });
                });
                ui.vertical(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.hyperlink_to(
                            egui::RichText::new("CLOEI")
                                .color(egui::Color32::from_rgb(30, 144, 255)),
                            "https://github.com/CLOEI",
                        );
                        ui.label("Created by:");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label("A passionate software developer");
                    });
                    ui.add_space(20.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label("Contributors:");
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.hyperlink_to(
                            egui::RichText::new("Foite")
                                .color(egui::Color32::from_rgb(30, 144, 255)),
                            "https://github.com/foite",
                        );
                        ui.hyperlink_to(
                            egui::RichText::new("bbangg")
                                .color(egui::Color32::from_rgb(30, 144, 255)),
                            "https://github.com/bbangg",
                        )
                    });
                });
                ui.end_row();
            });
    }
}
