use std::{
    fs::{self, File},
    io::Write,
};

use bot::Bot;
use eframe::egui::ViewportBuilder;
use gui::{
    add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase, navbar::Navbar,
};
use manager::Manager;
use types::config::{BotConfig, Config};

mod bot;
mod gui;
mod manager;
mod types;
mod utils;

fn init_config() {
    if !fs::metadata("config.json").is_ok() {
        let mut file = File::create("config.json").unwrap();
        let config = Config {
            bots: Vec::new(),
            timeout: 5,
            findpath_delay: 5,
        };
        let j = serde_json::to_string_pretty(&config).unwrap();
        file.write_all(j.as_bytes()).unwrap();
    }
}

fn main() {
    init_config();

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
    add_bot_dialog: AddBotDialog,
    manager: Manager,
    bots: Vec<BotConfig>,
    bot_menu: BotMenu,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut manager = Manager::new();
        let bots = utils::config::get_bots();
        for bot in bots.clone() {
            manager.add_bot(bot);
        }

        Self {
            navbar: Default::default(),
            item_database: Default::default(),
            add_bot_dialog: Default::default(),
            manager: manager,
            bots: bots,
            bot_menu: Default::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.bots = utils::config::get_bots();
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
