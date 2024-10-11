#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::{self, File},
    io::Write,
};
use std::sync::{Arc, RwLock};
use core::Bot;
use eframe::egui::ViewportBuilder;
use gui::{
    add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase, navbar::Navbar,
};
use types::{
    config::{Config},
};
use mlua::prelude::*;
use crate::gui::add_proxy_dialog::AddProxyDialog;
use crate::gui::proxy_list::ProxyList;
use crate::gui::settings::Settings;
use crate::manager::bot_manager::BotManager;
use crate::manager::proxy_manager::ProxyManager;
use crate::utils::config;

mod core;
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
            proxy: Vec::new(),
            timeout: 5,
            findpath_delay: 30,
            selected_bot: "".to_string(),
            game_version: "4.65".to_string(),
            use_alternate_server: false,
            dark_mode: true,
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
    add_proxy_dialog: AddProxyDialog,
    bot_manager: Arc<RwLock<BotManager>>,
    proxy_manager: Arc<RwLock<ProxyManager>>,
    proxy_list: ProxyList,
    settings: Settings,
    bot_menu: BotMenu,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let proxy_manager = Arc::new(RwLock::new(ProxyManager::new()));
        let mut bot_manager = Arc::new(RwLock::new(BotManager::new(proxy_manager.clone())));
        let bots = utils::config::get_bots();
        for bot in bots.clone() {
            bot_manager.write().unwrap().add_bot(bot);
        }

        Self {
            navbar: Default::default(),
            item_database: Default::default(),
            add_bot_dialog: Default::default(),
            add_proxy_dialog: Default::default(),
            bot_menu: Default::default(),
            proxy_list: Default::default(),
            settings: Settings {
                use_alternate: config::get_use_alternate_server(),
                timeout_delay: config::get_timeout(),
                findpath_delay: config::get_findpath_delay(),
                dark_mode: config::get_dark_mode(),
            },
            proxy_manager,
            bot_manager,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if self.settings.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui_extras::install_image_loaders(ctx);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.navbar
                .render(ui, &mut self.add_bot_dialog, &self.bot_manager);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.navbar.current_menu == "bots" {
                self.bot_menu.render(ui, &self.bot_manager);
            } else if self.navbar.current_menu == "item_database" {
                self.item_database.render(ui, &self.bot_manager, ctx);
            } else if self.navbar.current_menu == "proxy_list" {
                self.proxy_list.render(ui, &self.proxy_manager, &mut self.add_proxy_dialog, ctx);
            } else if self.navbar.current_menu == "settings" {
                self.settings.render(ui, ctx);
            } else {
                ui.label("Not implemented yet");
            }
        });
        self.add_bot_dialog.render(&mut self.bot_manager, ctx);
        self.add_proxy_dialog.render(&mut self.proxy_manager, ctx);
    }
}
