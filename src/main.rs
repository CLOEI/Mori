#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::gui::add_proxy_dialog::AddProxyDialog;
use crate::gui::proxy_list::ProxyList;
use crate::gui::settings::Settings;
use crate::manager::bot_manager::BotManager;
use crate::manager::proxy_manager::ProxyManager;
use crate::texture_manager::TextureManager;
use crate::utils::config;
use eframe::egui::ViewportBuilder;
use egui::{
    vec2, Button, CentralPanel, Id, PointerButton, RichText, Sense, UiBuilder, ViewportCommand,
};
use gui::{
    add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase, navbar::Navbar,
};
// use paris::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::{
    fs::{self, File},
    io::Write,
};
// use eframe::{HardwareAcceleration, Renderer};
use types::config::{Config};

mod core;
mod gui;
mod lua_register;
mod manager;
mod texture_manager;
mod types;
mod utils;

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
            game_version: "5.11".to_string(),
            use_alternate_server: false,
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
            .with_inner_size([850.0, 450.0])
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
    texture_manager: Arc<RwLock<TextureManager>>,
    proxy_list: ProxyList,
    settings: Settings,
    bot_menu: BotMenu,
    texture_loaded: Arc<AtomicBool>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let texture_manager = Arc::new(RwLock::new(TextureManager::new()));
        let proxy_manager = Arc::new(RwLock::new(ProxyManager::new()));
        let bot_manager = Arc::new(RwLock::new(BotManager::new(proxy_manager.clone())));
        let texture_loaded = Arc::new(AtomicBool::new(true));
        let bot_manager_clone = bot_manager.clone();

        // {
        //     let bot_manager_clone = bot_manager.clone();
        //     let texture_manager_clone = texture_manager.clone();
        //     let texture_loaded_clone = texture_loaded.clone();
        //     let egui_ctx = cc.egui_ctx.clone();
        //     let growtopia_path = dirs::data_local_dir().unwrap().join("Growtopia/game");
        //     if growtopia_path.exists() {
        //         info!("Found Growtopia at path: {:?}", growtopia_path);
        //         thread::spawn(move || {
        //             let mut texture_manager = texture_manager_clone.write().unwrap();
        //             texture_manager.load_textures(&egui_ctx, growtopia_path.as_path());
        //             texture_loaded_clone.store(true, Ordering::Release);
        //             let bots = config::get_bots();
        //             for bot in bots.clone() {
        //                 bot_manager_clone.write().unwrap().add_bot(bot);
        //             }
        //         });
        //     } else {
        //         texture_loaded_clone.store(true, Ordering::Release);
        //         let bots = config::get_bots();
        //         for bot in bots.clone() {
        //             bot_manager_clone.write().unwrap().add_bot(bot);
        //         }
        //         warn!("Growtopia not found");
        //     }
        // }

        let bots = config::get_bots();
        for bot in bots.clone() {
            bot_manager_clone.write().unwrap().add_bot(bot);
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
                captcha_provider: config::get_captcha_provider(),
                captcha_api_key: config::get_captcha_api_key(),
            },
            proxy_manager,
            bot_manager,
            texture_manager,
            texture_loaded,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui_extras::install_image_loaders(ctx);

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            rounding: 6.0.into(),
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
                    self.navbar
                        .render(ui);
                        // .render(ui, &mut self.add_bot_dialog, &self.bot_manager);
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
                        .add(Button::new(
                            RichText::new(egui_phosphor::variants::fill::POWER)
                                .size(button_height),
                        ))
                        .on_hover_text("Close the app")
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }

                    if ui
                        .add(Button::new(
                            RichText::new(egui_phosphor::variants::fill::CARET_DOWN)
                                .size(button_height),
                        ))
                        .on_hover_text("Minimize the window")
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
                    }
                },
            );

            let content_rect = {
                let mut rect = app_rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }
            .shrink(4.0);

            let mut content_ui = ui.new_child(UiBuilder::new().max_rect(content_rect));
            if self.texture_loaded.load(Ordering::Acquire) {
                match self.navbar.current_menu.as_str() {
                    "bots" => self.bot_menu.render(
                        &mut content_ui,
                        &mut self.add_bot_dialog,
                        &self.bot_manager,
                        &self.texture_manager,
                    ),
                    "item_database" => self.item_database.render(
                        &mut content_ui,
                        &self.bot_manager,
                        &self.texture_manager,
                        ctx,
                    ),
                    "proxy_list" => self.proxy_list.render(
                        &mut content_ui,
                        &self.proxy_manager,
                        &mut self.add_proxy_dialog,
                        ctx,
                    ),
                    "settings" => self.settings.render(&mut content_ui, ctx),
                    _ => {}
                }
            } else {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        ui.add_space(ui.available_height() / 2.0 - 25.0);
                        ui.vertical_centered(|ui| {
                            ui.add(egui::Label::new(
                                RichText::new(egui_phosphor::variants::fill::PUZZLE_PIECE).size(50.0),
                            ));
                            ui.add(egui::Label::new("Loading textures..."));
                            ui.add_space(8.0);
                            ui.add(egui::Spinner::default());
                        });
                        ui.add_space(ui.available_height() / 2.0 - 25.0);
                    },
                );
            }

            self.add_bot_dialog.render(&mut self.bot_manager, ctx);
            self.add_proxy_dialog.render(&mut self.proxy_manager, ctx);
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}
