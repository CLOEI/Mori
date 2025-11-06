#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    gui::{add_bot_dialog::AddBotDialog, bot_menu::BotMenu, item_database::ItemDatabase},
    manager::bot_manager::BotManager,
};
use eframe::egui::ViewportBuilder;
use egui::{
    Button, CentralPanel, Id, PointerButton, RichText, Sense, UiBuilder, ViewportCommand, vec2,
};
use gui::navbar::Navbar;
use std::sync::{Arc, RwLock};

mod gui;
mod manager;

fn main() {
    let options = eframe::NativeOptions {
        centered: true,
        viewport: ViewportBuilder::default()
            .with_title("Mori")
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../stuff/logo.png")[..])
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
    bot_manager: Arc<RwLock<BotManager>>,
    bot_menu: BotMenu,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_material_icons::initialize(&cc.egui_ctx);

        let bot_manager = Arc::new(RwLock::new(BotManager::new()));
        let bot_manager_clone = bot_manager.clone();

        Self {
            navbar: Default::default(),
            item_database: Default::default(),
            add_bot_dialog: Default::default(),
            bot_menu: Default::default(),
            bot_manager: bot_manager_clone,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui_extras::install_image_loaders(ctx);

        let panel_frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            corner_radius: 6.0.into(),
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

            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(title_bar_rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                |ui| {
                    ui.add_space(8.0);
                    ui.heading("Mori");
                    ui.separator();
                    self.navbar.render(ui);
                },
            );

            ui.scope_builder(
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
                            RichText::new(egui_material_icons::icons::ICON_POWER_SETTINGS_NEW)
                                .size(button_height),
                        ))
                        .on_hover_text("Close the app")
                        .clicked()
                    {
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    }

                    if ui
                        .add(Button::new(
                            RichText::new(egui_material_icons::icons::ICON_MINIMIZE)
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
            match self.navbar.current_menu.as_str() {
                "bots" => self.bot_menu.render(
                    &mut content_ui,
                    &mut self.add_bot_dialog,
                    &self.bot_manager,
                ),
                "item_database" => {
                    self.item_database
                        .render(&mut content_ui, &self.bot_manager, ctx)
                }
                _ => {}
            }

            self.add_bot_dialog.render(&mut self.bot_manager, ctx);
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}
