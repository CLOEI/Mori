#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs::{self, File},
    io::Write,
};
use std::sync::{Arc, RwLock};
use eframe::egui::ViewportBuilder;
use egui::{CentralPanel, ViewportCommand, UiBuilder, Id, Sense, vec2, PointerButton, Button, RichText};
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
            auto_collect: true,
            selected_bot: "".to_string(),
            game_version: "4.68".to_string(),
            use_alternate_server: false,
            dark_mode: true,
            captcha: Default::default(),
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
            .with_decorations(false)
            .with_transparent(true),
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
        egui_remixicon::add_to_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

        let proxy_manager = Arc::new(RwLock::new(ProxyManager::new()));
        let mut bot_manager = Arc::new(RwLock::new(BotManager::new(proxy_manager.clone())));
        let bots = config::get_bots();
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
                auto_collect: config::get_auto_collect(),
                dark_mode: config::get_dark_mode(),
            },
            proxy_manager,
            bot_manager,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if self.settings.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui_extras::install_image_loaders(ctx);

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            rounding: 5.0.into(),
            outer_margin: 0.5.into(),
            ..Default::default()
        };

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let app_rect = ui.max_rect();

            let title_bar_height = 40.0;
            let title_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.min.y + title_bar_height;
                rect
            };

            let painter = ui.painter();
            let title_bar_response = ui.interact(
                title_bar_rect,
                Id::new("title_bar"),
                Sense::click_and_drag(),
            );

            painter.line_segment(
                [
                    title_bar_rect.left_bottom() + vec2(1.0, 0.0),
                    title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
                ],
                ui.visuals().widgets.noninteractive.bg_stroke,
            );

            if title_bar_response.drag_started_by(PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }

            ui.allocate_new_ui(
                UiBuilder::new()
                    .max_rect(title_bar_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                |ui| {
                    ui.add_space(8.0);
                    ui.heading("Mori");
                    ui.separator();
                    self.navbar.render(ui, &mut self.add_bot_dialog, &self.bot_manager);
                },
            );

            ui.allocate_new_ui(
                UiBuilder::new()
                    .max_rect(title_bar_rect)
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
                |ui| {
                    let button_height = 12.0;
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.visuals_mut().button_frame = false;
                    ui.add_space(8.0);

                    if ui
                        .add(Button::new(RichText::new(egui_remixicon::icons::SHUT_DOWN_LINE).size(button_height)))
                        .on_hover_text("Close the app").clicked() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }

                    if ui
                        .add(Button::new(RichText::new(egui_remixicon::icons::ARROW_DROP_DOWN_LINE).size(button_height)))
                        .on_hover_text("Minimize the window").clicked() {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                    }
                },
            );

            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }.shrink(4.0);

            let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
            match self.navbar.current_menu.as_str() {
                "bots" => self.bot_menu.render(&mut content_ui, &self.bot_manager),
                "item_database" => self.item_database.render(&mut content_ui, &self.bot_manager, ctx),
                "proxy_list" => self.proxy_list.render(&mut content_ui, &self.proxy_manager, &mut self.add_proxy_dialog, ctx),
                "settings" => self.settings.render(&mut content_ui, ctx),
                _ => {}
            }
            self.add_bot_dialog.render(&mut self.bot_manager, ctx);
            self.add_proxy_dialog.render(&mut self.proxy_manager, ctx);
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}