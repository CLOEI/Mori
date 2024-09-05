#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::{self, File},
    io::Write,
};
use std::sync::Arc;
use crate::gui::inventory::Inventory;
use bot::Bot;
use eframe::egui::ViewportBuilder;
use gui::{
    add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase, navbar::Navbar,
    world_map::WorldMap,
};
use manager::Manager;
use types::{
    config::{Config},
};
use mlua::prelude::*;
use paris::error;
use parking_lot::RwLock;

mod bot;
mod gui;
mod manager;
mod types;
mod utils;
mod lua_register;

fn init_config() {
    if !fs::metadata("config.json").is_ok() {
        let mut file = File::create("config.json").unwrap();
        let config = Config {
            bots: Vec::new(),
            timeout: 5,
            findpath_delay: 30,
            selected_bot: "".to_string(),
            game_version: "4.64".to_string(),
        };
        let j = serde_json::to_string_pretty(&config).unwrap();
        file.write_all(j.as_bytes()).unwrap();
    }
}

fn main() {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        for deadlock in parking_lot::deadlock::check_deadlock() {
            for deadlock in deadlock {
                error!(
                    "Found a deadlock! {}:\n{:?}",
                    deadlock.thread_id(),
                    deadlock.backtrace()
                );
            }
        }
    });
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
            .with_resizable(false)
            .with_fullscreen(false)
            .with_maximize_button(false),
        ..Default::default()
    };
    let _ = eframe::run_native("Mori", options, Box::new(|cc| Ok(Box::new(App::new(cc)))));
}

struct App {
    navbar: Navbar,
    item_database: ItemDatabase,
    add_bot_dialog: AddBotDialog,
    manager: Arc<RwLock<Manager>>,
    bot_menu: BotMenu,
    world_map: WorldMap,
    inventory: Inventory,
    lua: Lua,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut manager = Arc::new(RwLock::new(Manager::new()));
        let lua = Lua::new();
        let bots = utils::config::get_bots();
        for bot in bots.clone() {
            manager.write().add_bot(bot);
        }

        lua_register::register(&lua, &manager);

        Self {
            navbar: Default::default(),
            item_database: Default::default(),
            add_bot_dialog: Default::default(),
            bot_menu: Default::default(),
            world_map: Default::default(),
            inventory: Default::default(),
            manager,
            lua,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.navbar
                .render(ui, &mut self.add_bot_dialog, &self.manager);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.navbar.current_menu == "bots" {
                self.bot_menu.render(ui, &self.manager);
            } else if self.navbar.current_menu == "inventory" {
                self.inventory.render(ui, &self.manager, ctx);
            } else if self.navbar.current_menu == "item_database" {
                self.item_database.render(ui, &self.manager, ctx);
            } else if self.navbar.current_menu == "world_map" {
                self.world_map.render(ui, &self.manager);
            } else {
                ui.label("Not implemented yet");
            }
        });
        self.add_bot_dialog.render(&mut self.manager, ctx);
    }
}
