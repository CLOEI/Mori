use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_egui::egui::ComboBox;
use egui_flex::{item, Flex};
use egui_virtual_list::VirtualList;
use gt_core::gtitem_r;
use gt_core::gtitem_r::structs::ItemDatabase;
use gt_core::Bot;
use gt_core::types::bot::LoginVia;

#[derive(Debug, PartialEq, Clone)]
enum LoginType {
    GOOGLE,
    APPLE,
    LTOKEN,
    LEGACY,
}

#[derive(Resource)]
struct UiState {
    selected_bot: usize,
    add_bot_window_open: bool,
    item_database_window_open: bool,
    settings_window_open: bool,
    item_database: Arc<RwLock<ItemDatabase>>,
    virtual_list: VirtualList,
    search_text: String,
    // Add bot form fields
    login_type: LoginType,
    ltoken_string: String,
    legacy_fields: [String; 2],
    // Bots list
    bots: Vec<Arc<Bot>>,
}

impl Default for UiState {
    fn default() -> Self {
        let item_database = if Path::new("items.dat").exists() {
            println!("ItemDatabase loaded");
            gtitem_r::load_from_file("items.dat").unwrap_or_else(|_| ItemDatabase::new())
        } else {
            ItemDatabase::new()
        };

        Self {
            selected_bot: 0,
            add_bot_window_open: false,
            item_database_window_open: false,
            settings_window_open: false,
            item_database: Arc::new(RwLock::new(item_database)),
            virtual_list: VirtualList::new(),
            search_text: String::new(),
            login_type: LoginType::LEGACY,
            ltoken_string: String::new(),
            legacy_fields: Default::default(),
            bots: Vec::new(),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(UiState {
            selected_bot: 0,
            ..Default::default()
        })
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, setup_ui_system)
        .run();
}

fn setup_camera_system(
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
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
    egui_material_icons::initialize(&ctx);

    let mut top = egui::TopBottomPanel::top("top_panel")
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Mori");
                let bot_count = ui_state.bots.len();
                let selected_text = if bot_count == 0 {
                    "No bots".to_string()
                } else {
                    format!("Bot {}", ui_state.selected_bot + 1)
                };

                ComboBox::from_id_salt("bot_selection")
                    .selected_text(&selected_text)
                    .show_ui(ui, |ui| {
                        for index in 0..bot_count {
                            if ui.selectable_value(&mut ui_state.selected_bot, index, format!("Bot {}", index + 1)).clicked() {
                                // Selection updated
                            }
                        }
                    });
                ui.separator();
                if ui.button(format!("{} Add bot", egui_material_icons::icons::ICON_ADD)).clicked() {
                    ui_state.add_bot_window_open = !ui_state.add_bot_window_open;
                }
                if ui.button(format!("{} Item Database", egui_material_icons::icons::ICON_DATABASE)).clicked() {
                    ui_state.item_database_window_open = !ui_state.item_database_window_open;
                }
                if ui.button(format!("{} Settings", egui_material_icons::icons::ICON_SETTINGS)).clicked() {
                    ui_state.settings_window_open = !ui_state.settings_window_open;
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
               Flex::new()
                   .w_full()
                   .align_items(egui_flex::FlexAlign::Center)
                   .justify(egui_flex::FlexJustify::SpaceBetween)
                   .show(ui, |flex| {
                       flex.add_ui(item(), |ui| {
                           ui.label("Created with ❤ by Cendy");
                       });
                       flex.add_ui(item(), |ui| {
                           ui.label(format!("{} Gems: {}", egui_material_icons::icons::ICON_DIAMOND, "0"));
                           ui.label(format!("{} Ping: {}", egui_material_icons::icons::ICON_WIFI, "0"));
                           if ui.button(format!("{} World info", egui_material_icons::icons::ICON_PUBLIC)).clicked() {}
                           if ui.button(format!("{} Inventory", egui_material_icons::icons::ICON_BACKPACK)).clicked() {}
                       });
                   })
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();

    let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!("{} Console", egui_material_icons::icons::ICON_TERMINAL));
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
            ui.vertical(|ui| {
                ui.label("Login Method:");
                ui.horizontal(|ui| {
                    let _ = ui.add_enabled(false, egui::Button::new("Google"));
                    let _ = ui.add_enabled(false, egui::Button::new("Apple"));
                    let _ = ui.add_enabled(false, egui::Button::new("LTOKEN"));
                    ui.selectable_value(&mut ui_state.login_type, LoginType::LEGACY, "Legacy");
                });

                ui.separator();

                match ui_state.login_type {
                    LoginType::GOOGLE => {
                        ui.label(egui::RichText::new("Google login - Coming soon")
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.label("No additional fields required.");
                    }
                    LoginType::APPLE => {
                        ui.label(egui::RichText::new("Apple login - Coming soon")
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.label("No additional fields required.");
                    }
                    LoginType::LTOKEN => {
                        ui.label(egui::RichText::new("LTOKEN login - Coming soon")
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.add_enabled_ui(false, |ui| {
                            ui.text_edit_singleline(&mut ui_state.ltoken_string);
                            ui.label(egui::RichText::new("Enter 4 values separated by colons (:)")
                                .size(12.0)
                                .color(egui::Color32::from_rgb(150, 150, 150)));
                        });
                    }
                    LoginType::LEGACY => {
                        ui.label("Legacy login requires 2 fields:");
                        ui.horizontal(|ui| {
                            ui.label("GrowID:");
                            ui.text_edit_singleline(&mut ui_state.legacy_fields[0]);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Password:");
                            ui.text_edit_singleline(&mut ui_state.legacy_fields[1]);
                        });
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Create Bot").clicked() {
                        let login_via = match ui_state.login_type {
                            LoginType::LEGACY => {
                                LoginVia::LEGACY([
                                    ui_state.legacy_fields[0].clone(),
                                    ui_state.legacy_fields[1].clone()
                                ])
                            }
                            LoginType::GOOGLE => LoginVia::GOOGLE,
                            LoginType::APPLE => LoginVia::APPLE,
                            LoginType::LTOKEN => {
                                let parts: Vec<String> = ui_state.ltoken_string
                                    .split(':')
                                    .map(|s| s.to_string())
                                    .collect();
                                if parts.len() == 4 {
                                    LoginVia::LTOKEN([
                                        parts[0].clone(),
                                        parts[1].clone(),
                                        parts[2].clone(),
                                        parts[3].clone()
                                    ])
                                } else {
                                    println!("Invalid LTOKEN format. Expected 4 values separated by colons.");
                                    return;
                                }
                            }
                        };

                        let item_database = Arc::clone(&ui_state.item_database);
                        let bot = Bot::new(login_via, None, item_database);

                        let bot_clone = Arc::clone(&bot);

                        thread::spawn(move || {
                            bot_clone.logon(None);
                        });

                        ui_state.bots.push(bot);

                        ui_state.add_bot_window_open = false;
                        ui_state.legacy_fields = Default::default();
                        ui_state.ltoken_string = String::new();
                        ui_state.login_type = LoginType::LEGACY;

                        println!("Bot created and added to bots list");
                    }
                    if ui.button("Cancel").clicked() {
                        ui_state.add_bot_window_open = false;
                    }
                });
            });
        });
    }

    if ui_state.item_database_window_open {
        egui::Window::new(format!("{} Item Database", egui_material_icons::icons::ICON_DATABASE))
            .default_size([950.0, 750.0])
            .resizable(true)
            .collapsible(false)
            .show(contexts.ctx_mut()?, |ui| {
                let (item_count, items) = {
                    let item_database = ui_state.item_database.read().unwrap();
                    let count = item_database.item_count;
                    let mut item_vec: Vec<_> = item_database.items.values().cloned().collect();
                    item_vec.sort_by_key(|item| item.id);
                    (count, item_vec)
                };

                // Header section with stats
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(16, 12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Total Items: {}", item_count))
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(180, 180, 180)));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button(format!("{} Close", egui_material_icons::icons::ICON_CLOSE)).clicked() {
                                            ui_state.item_database_window_open = false;
                                        }
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("Search:")
                                            .color(egui::Color32::from_rgb(180, 180, 180)));
                                        let response = ui.text_edit_singleline(&mut ui_state.search_text);
                                        if response.changed() {
                                            ui_state.virtual_list.reset();
                                        }
                                        if ui.button("Clear").clicked() {
                                            ui_state.search_text.clear();
                                            ui_state.virtual_list.reset();
                                        }
                                    });
                                });
                            });
                        });
                    });

                ui.add_space(8.0);

                if items.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("No items loaded")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 200, 100)));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Make sure items.dat is present and valid.")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.add_space(20.0);
                    });
                    return;
                }

                let filtered_items: Vec<_> = if ui_state.search_text.is_empty() {
                    items
                } else {
                    items.into_iter()
                        .filter(|item| {
                            item.name.to_lowercase().contains(&ui_state.search_text.to_lowercase()) ||
                            item.description.to_lowercase().contains(&ui_state.search_text.to_lowercase()) ||
                            format!("{}", item.id).contains(&ui_state.search_text)
                        })
                        .collect()
                };

                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(25, 25, 30))
                    .inner_margin(egui::Margin::symmetric(8, 8))
                    .outer_margin(egui::Margin::symmetric(0, 0))
                    .corner_radius(6.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui_state.virtual_list.ui_custom_layout(ui, filtered_items.len(), |ui, start_index| {
                        let item = &filtered_items[start_index];

                        let frame_color = egui::Color32::from_rgb(45, 45, 45);

                        egui::Frame::new()
                            .fill(frame_color)
                            .inner_margin(egui::Margin::symmetric(16, 12))
                            .outer_margin(egui::Margin::symmetric(0, 4))
                            .corner_radius(8.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("ID: {}", item.id))
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if item.rarity > 0 {
                                            ui.label(egui::RichText::new(format!("Rarity: {}", item.rarity))
                                                .strong());
                                        }
                                    });
                                });

                                ui.add_space(4.0);

                                ui.label(egui::RichText::new(&item.name)
                                    .size(18.0)
                                    .color(egui::Color32::WHITE)
                                    .strong());

                                ui.add_space(6.0);

                                if !item.description.is_empty() {
                                    ui.label(egui::RichText::new(&item.description)
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(180, 180, 180))
                                        .italics());
                                    ui.add_space(8.0);
                                }

                                let mut flags = Vec::new();
                                if item.flags.flippable { flags.push((egui_material_icons::icons::ICON_FLIP, "Flippable", egui::Color32::from_rgb(100, 200, 255))); }
                                if item.flags.editable { flags.push((egui_material_icons::icons::ICON_CREATE, "Editable", egui::Color32::from_rgb(255, 200, 100))); }
                                if item.flags.seedless { flags.push(("🌱", "Seedless", egui::Color32::from_rgb(100, 255, 150))); }
                                if item.flags.permanent { flags.push((egui_material_icons::icons::ICON_BOLT, "Permanent", egui::Color32::from_rgb(255, 255, 100))); }
                                if item.flags.dropless { flags.push((egui_material_icons::icons::ICON_BLOCK, "Dropless", egui::Color32::from_rgb(255, 100, 100))); }
                                if item.flags.no_self { flags.push((egui_material_icons::icons::ICON_PERSON, "No Self", egui::Color32::from_rgb(200, 150, 255))); }
                                if item.flags.no_shadow { flags.push((egui_material_icons::icons::ICON_TONALITY, "No Shadow", egui::Color32::from_rgb(150, 150, 150))); }
                                if item.flags.world_locked { flags.push((egui_material_icons::icons::ICON_LOCK, "World Locked", egui::Color32::from_rgb(255, 150, 150))); }
                                if item.flags.beta { flags.push((egui_material_icons::icons::ICON_SCIENCE, "Beta", egui::Color32::from_rgb(150, 255, 150))); }
                                if item.flags.auto_pickup { flags.push((egui_material_icons::icons::ICON_SMART_TOY, "Auto Pickup", egui::Color32::from_rgb(255, 200, 200))); }
                                if item.flags.mod_flag { flags.push(("🛠", "Mod Flag", egui::Color32::from_rgb(200, 200, 255))); }
                                if item.flags.random_grow { flags.push((egui_material_icons::icons::ICON_CASINO, "Random Grow", egui::Color32::from_rgb(255, 150, 255))); }
                                if item.flags.public { flags.push((egui_material_icons::icons::ICON_PUBLIC, "Public", egui::Color32::from_rgb(150, 255, 255))); }
                                if item.flags.foreground { flags.push((egui_material_icons::icons::ICON_SQUARE, "Foreground", egui::Color32::from_rgb(255, 255, 150))); }
                                if item.flags.holiday { flags.push((egui_material_icons::icons::ICON_PARK, "Holiday", egui::Color32::from_rgb(255, 100, 200))); }
                                if item.flags.untradeable { flags.push((egui_material_icons::icons::ICON_BLOCK, "Untradeable", egui::Color32::from_rgb(255, 100, 100))); }

                                if !flags.is_empty() {
                                    ui.separator();
                                    ui.add_space(4.0);

                                    ui.horizontal_wrapped(|ui| {
                                        for (icon, name, color) in flags {
                                            ui.label(egui::RichText::new(format!("{} {}", icon, name))
                                                .size(11.0)
                                                .color(color)
                                                .strong());
                                            ui.add_space(8.0);
                                        }
                                    });
                                }
                            });

                                    1
                                });
                            });
                    });
            });
    }

    if ui_state.settings_window_open {
        egui::Window::new("Settings").show(contexts.ctx_mut()?, |ui| {
            ui.label("Settings will be here.");
        });
    }

    Ok(())
}