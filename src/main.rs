mod bot;
mod manager;
mod types;
mod utils;

use eframe::egui;
use manager::Manager;
use spdlog::prelude::*;

fn main() {
    let mut manager = match Manager::new() {
        Ok(manager) => manager,
        Err(err) => {
            error!("Error: {}", err);
            return;
        }
    };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Mori",
        options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    );
}
#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
