use std::path::Path;
use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_egui::egui::ComboBox;
use gt_core::gtitem_r;
use gt_core::gtitem_r::structs::ItemDatabase;

#[derive(Resource)]
struct UiState {
    selected_bot: &'static str,
    add_bot_window_open: bool,
    item_database_window_open: bool,
    settings_window_open: bool,
    item_database: Arc<Mutex<ItemDatabase>>
}

impl Default for UiState {
    fn default() -> Self {
        let item_database = if Path::new("items.dat").exists() {
            gtitem_r::load_from_file("items.dat").unwrap_or_else(|_| ItemDatabase::new())
        } else {
            ItemDatabase::new()
        };

        Self {
            selected_bot: "bot-1",
            add_bot_window_open: false,
            item_database_window_open: false,
            settings_window_open: false,
            item_database: Arc::new(Mutex::new(item_database))
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(UiState {
            selected_bot: "bot-1",
            ..Default::default()
        })
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, setup_ui_system)
        .run();
}

fn setup_camera_system(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn(Camera2d);

    commands.spawn((
        PrimaryEguiContext,
        Camera2d,
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        }
    ));
}

fn setup_ui_system(
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut ui_state: ResMut<UiState>
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let mut top = egui::TopBottomPanel::top("top_panel")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Mori");
                ComboBox::from_id_salt("bot_selection")
                    .selected_text(ui_state.selected_bot)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ui_state.selected_bot, "bot-1", "Bot 1");
                        ui.selectable_value(&mut ui_state.selected_bot, "bot-2", "Bot 2");
                        ui.selectable_value(&mut ui_state.selected_bot, "bot-3", "Bot 3");
                    });
                ui.separator();
                if ui.button("Add bot").clicked() {
                    ui_state.add_bot_window_open = !ui_state.add_bot_window_open;
                }
                if ui.button("Item Database").clicked() {
                    ui_state.item_database_window_open = !ui_state.item_database_window_open;
                }
                if ui.button("Settings").clicked() {
                    ui_state.settings_window_open = !ui_state.settings_window_open;
                }
            });
            ui.separator();
            ui.label("Created with ❤ by Cendy");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();

    let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Console");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();

    top *= window.scale_factor();
    bottom *= window.scale_factor();

    let pos = UVec2::new(0, top as u32);
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(0, bottom as u32);

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    if ui_state.add_bot_window_open {
        egui::Window::new("Add bot").show(contexts.ctx_mut()?, |ui| {
            ui.label("List of bots will be here.");
        });
    }

    if ui_state.item_database_window_open {
        egui::Window::new("Item Database").show(contexts.ctx_mut()?, |ui| {
            ui.label("Item database will be here.");
        });
    }

    if ui_state.settings_window_open {
        egui::Window::new("Settings").show(contexts.ctx_mut()?, |ui| {
            ui.label("Settings will be here.");
        });
    }

    Ok(())
}