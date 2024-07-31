mod bot;
mod gui;
mod manager;
mod types;
mod utils;

use std::fs;

use eframe::egui::{self, include_image, IconData, ViewportBuilder};
use gui::{
    add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase, navbar::Navbar,
};
use manager::Manager;
use serde::{Deserialize, Serialize};
use types::e_login_method::ELoginMethod;

#[derive(Serialize, Deserialize)]
struct Data {
    bots: Vec<Bot>,
}

#[derive(Serialize, Deserialize)]
struct Bot {
    username: String,
    password: String,
    code: String,
    method: ELoginMethod,
}

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: ViewportBuilder::default()
            .with_title("Mori")
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/logo.png")[..])
                    .expect("Failed to load icon"),
            )
            .with_inner_size([800.0, 400.0])
            .with_resizable(false),
        ..Default::default()
    };
    let _ = eframe::run_native("Mori", options, Box::new(|cc| Ok(Box::new(App::new(cc)))));
}

struct App {
    navbar: Navbar,
    item_database: ItemDatabase,
    manager: Manager,
    add_bot_dialog: AddBotDialog,
    bots: Vec<Bot>,
    bot_menu: BotMenu,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut manager = Manager::new().unwrap();

        let data = match fs::read_to_string("data.json") {
            Ok(data) => data,
            Err(_) => {
                let data = Data { bots: vec![] };
                let json = serde_json::to_string_pretty(&data).unwrap();
                fs::write("data.json", &json).unwrap();
                json
            }
        };
        let json = serde_json::from_str::<Data>(&data).unwrap();
        for bot in &json.bots {
            manager.add_bot(
                bot.username.clone(),
                bot.password.clone(),
                bot.code.clone(),
                bot.method.clone(),
            );
        }

        Self {
            navbar: Default::default(),
            item_database: Default::default(),
            manager,
            add_bot_dialog: Default::default(),
            bots: json.bots,
            bot_menu: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.navbar.render(ui, &mut self.add_bot_dialog);
            ui.separator();
            if self.navbar.current_menu == "bots" {
                self.bot_menu.render(ui, &self.bots, &self.manager);
            } else if self.navbar.current_menu == "item_database" {
                self.item_database.render(ui, &mut self.manager, ctx);
            } else {
                ui.label("Not implemented yet");
            }
        });
        self.add_bot_dialog.render(&mut self.manager, ctx);
    }
}
