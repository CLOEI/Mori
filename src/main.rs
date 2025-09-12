mod token;

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
use gt_core::gtworld_r;
use gt_core::{Bot, Socks5Config};
use gt_core::types::bot::LoginVia;
use std::net::ToSocketAddrs;

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
    inventory_window_open: bool,
    world_info_window_open: bool,
    settings_window_open: bool,
    item_database: Arc<RwLock<ItemDatabase>>,
    virtual_list: VirtualList,
    search_text: String,
    // Add bot form fields
    login_type: LoginType,
    ltoken_string: String,
    legacy_fields: [String; 2],
    socks5_string: String,
    // Bots list
    bots: Vec<Arc<Bot>>,
}

impl Default for UiState {
    fn default() -> Self {
        let item_database = if Path::new("items.dat").exists() {
            gtitem_r::load_from_file("items.dat").unwrap_or_else(|_| ItemDatabase::new())
        } else {
            ItemDatabase::new()
        };

        Self {
            selected_bot: 0,
            add_bot_window_open: false,
            item_database_window_open: false,
            inventory_window_open: false,
            world_info_window_open: false,
            settings_window_open: false,
            item_database: Arc::new(RwLock::new(item_database)),
            virtual_list: VirtualList::new(),
            search_text: String::new(),
            login_type: LoginType::LEGACY,
            ltoken_string: String::new(),
            legacy_fields: Default::default(),
            socks5_string: String::new(),
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
            socks5_string: String::new(),
            inventory_window_open: false,
            world_info_window_open: false,
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
                } else if ui_state.selected_bot < bot_count {
                    let bot = &ui_state.bots[ui_state.selected_bot];
                    if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
                        if let Some(login_info) = login_info_lock.as_ref() {
                            if !login_info.tank_id_name.is_empty() {
                                login_info.tank_id_name.clone()
                            } else {
                                format!("Bot {}", ui_state.selected_bot + 1)
                            }
                        } else {
                            format!("Bot {}", ui_state.selected_bot + 1)
                        }
                    } else {
                        format!("Bot {}", ui_state.selected_bot + 1)
                    }
                } else {
                    format!("Bot {}", ui_state.selected_bot + 1)
                };

                ComboBox::from_id_salt("bot_selection")
                    .selected_text(&selected_text)
                    .show_ui(ui, |ui| {
                        for index in 0..bot_count {
                            let bot = &ui_state.bots[index];
                            let bot_label = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
                                if let Some(login_info) = login_info_lock.as_ref() {
                                    if !login_info.tank_id_name.is_empty() {
                                        login_info.tank_id_name.clone()
                                    } else {
                                        format!("Bot {}", index + 1)
                                    }
                                } else {
                                    format!("Bot {}", index + 1)
                                }
                            } else {
                                format!("Bot {}", index + 1)
                            };
                            
                            if ui.selectable_value(&mut ui_state.selected_bot, index, bot_label).clicked() {
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
                           let current_bot = if !ui_state.bots.is_empty() && ui_state.selected_bot < ui_state.bots.len() {
                               Some(&ui_state.bots[ui_state.selected_bot])
                           } else {
                               None
                           };
                           
                           let gems = current_bot
                               .map(|bot| bot.gems.load(std::sync::atomic::Ordering::Relaxed).to_string())
                               .unwrap_or_else(|| "0".to_string());
                           
                           let ping = current_bot
                               .map(|bot| bot.ping.load(std::sync::atomic::Ordering::Relaxed).to_string())
                               .unwrap_or_else(|| "0".to_string());
                           
                           ui.label(format!("{} Gems: {}", egui_material_icons::icons::ICON_DIAMOND, gems));
                           ui.label(format!("{} Ping: {}ms", egui_material_icons::icons::ICON_WIFI, ping));
                           if ui.button(format!("{} World info", egui_material_icons::icons::ICON_PUBLIC)).clicked() {
                               ui_state.world_info_window_open = !ui_state.world_info_window_open;
                           }
                           if ui.button(format!("{} Inventory", egui_material_icons::icons::ICON_BACKPACK)).clicked() {
                               ui_state.inventory_window_open = !ui_state.inventory_window_open;
                           }
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
                    ui.selectable_value(&mut ui_state.login_type, LoginType::GOOGLE, "Google");
                    ui.selectable_value(&mut ui_state.login_type, LoginType::APPLE, "Apple");
                    ui.selectable_value(&mut ui_state.login_type, LoginType::LTOKEN, "LTOKEN");
                    ui.selectable_value(&mut ui_state.login_type, LoginType::LEGACY, "Legacy");
                });

                ui.separator();

                match ui_state.login_type {
                    LoginType::GOOGLE => {
                        ui.label("Google login - Click connect to authenticate");
                        if ui.button("Connect Google").clicked() {
                            let item_database = Arc::clone(&ui_state.item_database);
                            let bot = Bot::new(LoginVia::GOOGLE, Some(Box::new(token::fetch)), item_database, None);

                            let bot_clone = Arc::clone(&bot);

                            thread::spawn(move || {
                                bot_clone.logon(None);
                            });

                            ui_state.bots.push(bot);

                            ui_state.add_bot_window_open = false;
                            ui_state.legacy_fields = Default::default();
                            ui_state.ltoken_string = String::new();
                            ui_state.socks5_string = String::new();
                            ui_state.login_type = LoginType::LEGACY;

                            println!("Google bot created and added to bots list");
                        }
                    }
                    LoginType::APPLE => {
                        ui.label("Apple login - Click connect to authenticate");
                        if ui.button("Connect Apple").clicked() {
                            let item_database = Arc::clone(&ui_state.item_database);
                            let bot = Bot::new(LoginVia::APPLE, Some(Box::new(token::fetch)), item_database, None);

                            let bot_clone = Arc::clone(&bot);

                            thread::spawn(move || {
                                bot_clone.logon(None);
                            });

                            ui_state.bots.push(bot);

                            ui_state.add_bot_window_open = false;
                            ui_state.legacy_fields = Default::default();
                            ui_state.ltoken_string = String::new();
                            ui_state.socks5_string = String::new();
                            ui_state.login_type = LoginType::LEGACY;

                            println!("Apple bot created and added to bots list");
                        }
                    }
                    LoginType::LTOKEN => {
                        ui.label("LTOKEN login - Enter your token:");
                        ui.text_edit_singleline(&mut ui_state.ltoken_string);
                        ui.label(egui::RichText::new("Enter 4 values separated by colons (:)")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        
                        if ui.button("Create LTOKEN Bot").clicked() {
                            let parts: Vec<String> = ui_state.ltoken_string
                                .split(':')
                                .map(|s| s.to_string())
                                .collect();
                            if parts.len() == 4 {
                                let login_via = LoginVia::LTOKEN([
                                    parts[0].clone(),
                                    parts[1].clone(),
                                    parts[2].clone(),
                                    parts[3].clone()
                                ]);

                                let item_database = Arc::clone(&ui_state.item_database);
                                let bot = Bot::new(login_via, None, item_database, None);

                                let bot_clone = Arc::clone(&bot);

                                thread::spawn(move || {
                                    bot_clone.logon(None);
                                });

                                ui_state.bots.push(bot);

                                ui_state.add_bot_window_open = false;
                                ui_state.legacy_fields = Default::default();
                                ui_state.ltoken_string = String::new();
                                ui_state.login_type = LoginType::LEGACY;

                                println!("LTOKEN bot created and added to bots list");
                            } else {
                                println!("Invalid LTOKEN format. Expected 4 values separated by colons.");
                            }
                        }
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
                        
                        ui.separator();
                        ui.label("SOCKS5 Proxy (optional):");
                        ui.horizontal(|ui| {
                            ui.label("Proxy:");
                            ui.text_edit_singleline(&mut ui_state.socks5_string);
                        });
                        ui.label(egui::RichText::new("Format: host:port or host:port:username:password")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        
                        if ui.button("Create Legacy Bot").clicked() {
                            if !ui_state.legacy_fields[0].is_empty() && !ui_state.legacy_fields[1].is_empty() {
                                let login_via = LoginVia::LEGACY([
                                    ui_state.legacy_fields[0].clone(),
                                    ui_state.legacy_fields[1].clone()
                                ]);

                                let socks5_config = if !ui_state.socks5_string.is_empty() {
                                    let parts: Vec<&str> = ui_state.socks5_string.split(':').collect();
                                    match parts.len() {
                                        2 => {
                                            // host:port format (no authentication)
                                            let host = parts[0];
                                            let port = parts[1];
                                            
                                            match format!("{}:{}", host, port).to_socket_addrs() {
                                                Ok(mut addrs) => {
                                                    if let Some(proxy_addr) = addrs.next() {
                                                        Some(Socks5Config {
                                                            proxy_addr,
                                                            username: None,
                                                            password: None,
                                                        })
                                                    } else {
                                                        println!("Could not resolve SOCKS5 proxy address: {}:{}", host, port);
                                                        None
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("Failed to resolve SOCKS5 proxy address {}:{}: {}", host, port, e);
                                                    None
                                                }
                                            }
                                        }
                                        4 => {
                                            // host:port:username:password format (with authentication)
                                            let host = parts[0];
                                            let port = parts[1];
                                            
                                            match format!("{}:{}", host, port).to_socket_addrs() {
                                                Ok(mut addrs) => {
                                                    if let Some(proxy_addr) = addrs.next() {
                                                        Some(Socks5Config {
                                                            proxy_addr,
                                                            username: Some(parts[2].to_string()),
                                                            password: Some(parts[3].to_string()),
                                                        })
                                                    } else {
                                                        println!("Could not resolve SOCKS5 proxy address: {}:{}", host, port);
                                                        None
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("Failed to resolve SOCKS5 proxy address {}:{}: {}", host, port, e);
                                                    None
                                                }
                                            }
                                        }
                                        _ => {
                                            println!("Invalid SOCKS5 format. Expected: host:port or host:port:username:password");
                                            None
                                        }
                                    }
                                } else {
                                    None
                                };

                                let item_database = Arc::clone(&ui_state.item_database);
                                let bot = Bot::new(login_via, None, item_database, socks5_config);

                                let bot_clone = Arc::clone(&bot);

                                thread::spawn(move || {
                                    bot_clone.logon(None);
                                });

                                ui_state.bots.push(bot);

                                ui_state.add_bot_window_open = false;
                                ui_state.legacy_fields = Default::default();
                                ui_state.ltoken_string = String::new();
                                ui_state.socks5_string = String::new();
                                ui_state.login_type = LoginType::LEGACY;

                                println!("Legacy bot created and added to bots list");
                            } else {
                                println!("Please fill in both GrowID and Password fields.");
                            }
                        }
                    }
                }

                ui.separator();
                ui.horizontal(|ui| {
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

    if ui_state.inventory_window_open {
        let current_bot = if !ui_state.bots.is_empty() && ui_state.selected_bot < ui_state.bots.len() {
            Some(&ui_state.bots[ui_state.selected_bot])
        } else {
            None
        };
        
        let selected_bot_index = ui_state.selected_bot;
        let item_database = Arc::clone(&ui_state.item_database);

        let mut close_inventory = false;
        
        egui::Window::new(format!("{} Inventory", egui_material_icons::icons::ICON_BACKPACK))
            .default_size([600.0, 500.0])
            .resizable(true)
            .collapsible(false)
            .show(contexts.ctx_mut()?, |ui| {
                if let Some(bot) = current_bot {
                    let bot_name = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
                        if let Some(login_info) = login_info_lock.as_ref() {
                            if !login_info.tank_id_name.is_empty() {
                                login_info.tank_id_name.clone()
                            } else {
                                format!("Bot {}", selected_bot_index + 1)
                            }
                        } else {
                            format!("Bot {}", selected_bot_index + 1)
                        }
                    } else {
                        format!("Bot {}", selected_bot_index + 1)
                    };

                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("Inventory - {}", bot_name))
                                    .size(16.0)
                                    .color(egui::Color32::WHITE)
                                    .strong());

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(format!("{} Close", egui_material_icons::icons::ICON_CLOSE)).clicked() {
                                        close_inventory = true;
                                    }
                                });
                            });
                        });

                    ui.add_space(8.0);

                    let inventory_data = if let Ok(inventory_lock) = bot.inventory.try_lock() {
                        Some((inventory_lock.size, inventory_lock.item_count, inventory_lock.items.clone()))
                    } else {
                        None
                    };

                    if let Some((size, item_count, items)) = inventory_data {
                        ui.horizontal(|ui| {
                            ui.label(format!("Size: {}", size));
                            ui.separator();
                            ui.label(format!("Items: {} / {}", item_count, size));
                        });

                        ui.add_space(8.0);

                        if items.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("Inventory is empty")
                                    .size(18.0)
                                    .color(egui::Color32::from_rgb(200, 200, 200)));
                                ui.add_space(20.0);
                            });
                        } else {
                            let item_db = item_database.read().unwrap();
                            
                            let mut sorted_items: Vec<_> = items.iter().collect();
                            sorted_items.sort_by_key(|(id, _)| *id);

                            egui::Frame::new()
                                .fill(egui::Color32::from_rgb(25, 25, 30))
                                .inner_margin(egui::Margin::symmetric(8, 8))
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            
                                            egui::Grid::new("inventory_grid")
                                                .num_columns(4)
                                                .spacing([10.0, 8.0])
                                                .show(ui, |ui| {
                                                    // Header
                                                    ui.label(egui::RichText::new("ID")
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                                    ui.label(egui::RichText::new("Item Name")
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                                    ui.label(egui::RichText::new("Amount")
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                                    ui.label(egui::RichText::new("Flag")
                                                        .strong()
                                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                                    ui.end_row();

                                                    ui.separator();
                                                    ui.separator();
                                                    ui.separator();
                                                    ui.separator();
                                                    ui.end_row();

                                                    // Items
                                                    for (&item_id, inventory_item) in sorted_items {
                                                        let item_name = item_db.items.get(&(item_id as u32))
                                                            .map(|item| item.name.as_str())
                                                            .unwrap_or("Unknown Item");

                                                        ui.label(egui::RichText::new(format!("{}", item_id))
                                                            .color(egui::Color32::from_rgb(150, 150, 255)));
                                                        ui.label(egui::RichText::new(item_name)
                                                            .color(egui::Color32::WHITE));
                                                        ui.label(egui::RichText::new(format!("{}", inventory_item.amount))
                                                            .color(egui::Color32::from_rgb(100, 255, 100)));
                                                        ui.label(egui::RichText::new(format!("{}", inventory_item.flag))
                                                            .color(egui::Color32::from_rgb(200, 200, 200)));
                                                        ui.end_row();
                                                    }
                                                });
                                        });
                                });
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("Unable to access inventory")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(255, 200, 100)));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Inventory might be locked or not loaded yet.")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(150, 150, 150)));
                            ui.add_space(20.0);
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("No bot selected")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 200, 100)));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Please select a bot to view its inventory.")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.add_space(20.0);
                    });
                }
            });
        
        if close_inventory {
            ui_state.inventory_window_open = false;
        }
    }

    if ui_state.world_info_window_open {
        let current_bot = if !ui_state.bots.is_empty() && ui_state.selected_bot < ui_state.bots.len() {
            Some(&ui_state.bots[ui_state.selected_bot])
        } else {
            None
        };
        
        let selected_bot_index = ui_state.selected_bot;
        let item_database = Arc::clone(&ui_state.item_database);

        let mut close_world_info = false;
        
        egui::Window::new(format!("{} World Info", egui_material_icons::icons::ICON_PUBLIC))
            .default_size([700.0, 600.0])
            .resizable(true)
            .collapsible(false)
            .show(contexts.ctx_mut()?, |ui| {
                if let Some(bot) = current_bot {
                    let bot_name = if let Ok(login_info_lock) = bot.info.login_info.try_lock() {
                        if let Some(login_info) = login_info_lock.as_ref() {
                            if !login_info.tank_id_name.is_empty() {
                                login_info.tank_id_name.clone()
                            } else {
                                format!("Bot {}", selected_bot_index + 1)
                            }
                        } else {
                            format!("Bot {}", selected_bot_index + 1)
                        }
                    } else {
                        format!("Bot {}", selected_bot_index + 1)
                    };

                    // Header section
                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("World Info - {}", bot_name))
                                    .size(16.0)
                                    .color(egui::Color32::WHITE)
                                    .strong());

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(format!("{} Close", egui_material_icons::icons::ICON_CLOSE)).clicked() {
                                        close_world_info = true;
                                    }
                                });
                            });
                        });

                    ui.add_space(8.0);

                    // Get world data and players
                    let world_data = if let Ok(world_lock) = bot.world.data.try_lock() {
                        Some((
                            world_lock.name.clone(),
                            world_lock.width,
                            world_lock.height,
                            world_lock.tiles.clone()
                        ))
                    } else {
                        None
                    };

                    let players_data = if let Ok(players_lock) = bot.world.players.try_lock() {
                        Some(players_lock.clone())
                    } else {
                        None
                    };

                    if let Some((world_name, width, height, tiles)) = world_data {
                        // World basic info
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(35, 35, 40))
                            .inner_margin(egui::Margin::symmetric(12, 10))
                            .corner_radius(6.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("World Name:")
                                        .strong()
                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                    ui.label(egui::RichText::new(&world_name)
                                        .color(egui::Color32::WHITE)
                                        .strong());
                                    
                                    ui.separator();
                                    
                                    ui.label(egui::RichText::new("Dimensions:")
                                        .strong()
                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                    ui.label(egui::RichText::new(format!("{}x{}", width, height))
                                        .color(egui::Color32::WHITE));
                                        
                                    ui.separator();
                                    
                                    ui.label(egui::RichText::new("Total Tiles:")
                                        .strong()
                                        .color(egui::Color32::from_rgb(200, 200, 200)));
                                    ui.label(egui::RichText::new(format!("{}", width * height))
                                        .color(egui::Color32::WHITE));
                                });
                            });

                        ui.add_space(8.0);

                        // Players section
                        if let Some(players) = players_data {
                            egui::CollapsingHeader::new(format!("{} Players ({})", egui_material_icons::icons::ICON_PEOPLE, players.len()))
                                .id_source("world_info_players")
                                .default_open(true)
                                .show(ui, |ui| {
                                    if players.is_empty() {
                                        ui.label(egui::RichText::new("No players in world")
                                            .color(egui::Color32::from_rgb(150, 150, 150)));
                                    } else {
                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgb(25, 25, 30))
                                            .inner_margin(egui::Margin::symmetric(8, 8))
                                            .corner_radius(6.0)
                                            .show(ui, |ui| {
                                                egui::ScrollArea::vertical()
                                                    .id_source("players_scroll")
                                                    .max_height(200.0)
                                                    .show(ui, |ui| {
                                                        for (net_id, player) in &players {
                                                            ui.horizontal(|ui| {
                                                                ui.label(egui::RichText::new(format!("NetID: {}", net_id))
                                                                    .color(egui::Color32::from_rgb(150, 150, 255)));
                                                                ui.separator();
                                                                ui.label(egui::RichText::new(&player.name)
                                                                    .color(egui::Color32::WHITE));
                                                                ui.separator();
                                                                ui.label(egui::RichText::new(format!("Pos: ({:.0}, {:.0})", player.position.0, player.position.1))
                                                                    .color(egui::Color32::from_rgb(100, 255, 100)));
                                                            });
                                                        }
                                                    });
                                            });
                                    }
                                });
                        }

                        ui.add_space(8.0);

                        // Tile counts section
                        egui::CollapsingHeader::new(format!("{} Tile Counts", egui_material_icons::icons::ICON_GRID_VIEW))
                            .id_source("world_info_tiles")
                            .default_open(true)
                            .show(ui, |ui| {
                                if tiles.is_empty() {
                                    ui.label(egui::RichText::new("No tile data available")
                                        .color(egui::Color32::from_rgb(150, 150, 150)));
                                } else {
                                    // Aggregate tile counts
                                    let mut foreground_counts = std::collections::HashMap::new();
                                    let mut background_counts = std::collections::HashMap::new();

                                    for tile in &tiles {
                                        if tile.foreground_item_id != 0 {
                                            *foreground_counts.entry(tile.foreground_item_id).or_insert(0u32) += 1;
                                        }
                                        if tile.background_item_id != 0 {
                                            *background_counts.entry(tile.background_item_id).or_insert(0u32) += 1;
                                        }
                                    }

                                    let item_db = item_database.read().unwrap();

                                    // Foreground tiles
                                    if !foreground_counts.is_empty() {
                                        ui.label(egui::RichText::new("Foreground Tiles:")
                                            .strong()
                                            .color(egui::Color32::from_rgb(255, 200, 100)));

                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgb(25, 25, 30))
                                            .inner_margin(egui::Margin::symmetric(8, 8))
                                            .corner_radius(6.0)
                                            .show(ui, |ui| {
                                                egui::ScrollArea::vertical()
                                                    .id_source("foreground_tiles_scroll")
                                                    .max_height(200.0)
                                                    .show(ui, |ui| {
                                                        // Sort by ID for consistent ordering
                                                        let mut sorted_fg: Vec<_> = foreground_counts.iter().collect();
                                                        sorted_fg.sort_by_key(|(id, _)| *id);

                                                        egui::Grid::new("foreground_grid")
                                                            .num_columns(3)
                                                            .spacing([10.0, 4.0])
                                                            .show(ui, |ui| {
                                                                for (&item_id, &count) in sorted_fg {
                                                                    let item_name = item_db.items.get(&(item_id as u32))
                                                                        .map(|item| item.name.as_str())
                                                                        .unwrap_or("Unknown Item");

                                                                    ui.label(egui::RichText::new(format!("{}", count))
                                                                        .color(egui::Color32::from_rgb(100, 255, 100))
                                                                        .strong());
                                                                    ui.label(egui::RichText::new(item_name)
                                                                        .color(egui::Color32::WHITE));
                                                                    ui.label(egui::RichText::new(format!("(ID: {})", item_id))
                                                                        .color(egui::Color32::from_rgb(150, 150, 255))
                                                                        .small());
                                                                    ui.end_row();
                                                                }
                                                            });
                                                    });
                                            });

                                        ui.add_space(8.0);
                                    }

                                    // Background tiles
                                    if !background_counts.is_empty() {
                                        ui.label(egui::RichText::new("Background Tiles:")
                                            .strong()
                                            .color(egui::Color32::from_rgb(200, 255, 200)));

                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgb(25, 25, 30))
                                            .inner_margin(egui::Margin::symmetric(8, 8))
                                            .corner_radius(6.0)
                                            .show(ui, |ui| {
                                                egui::ScrollArea::vertical()
                                                    .id_source("background_tiles_scroll")
                                                    .max_height(200.0)
                                                    .show(ui, |ui| {
                                                        // Sort by ID for consistent ordering
                                                        let mut sorted_bg: Vec<_> = background_counts.iter().collect();
                                                        sorted_bg.sort_by_key(|(id, _)| *id);

                                                        egui::Grid::new("background_grid")
                                                            .num_columns(3)
                                                            .spacing([10.0, 4.0])
                                                            .show(ui, |ui| {
                                                                for (&item_id, &count) in sorted_bg {
                                                                    let item_name = item_db.items.get(&(item_id as u32))
                                                                        .map(|item| item.name.as_str())
                                                                        .unwrap_or("Unknown Item");

                                                                    ui.label(egui::RichText::new(format!("{}", count))
                                                                        .color(egui::Color32::from_rgb(100, 255, 100))
                                                                        .strong());
                                                                    ui.label(egui::RichText::new(item_name)
                                                                        .color(egui::Color32::WHITE));
                                                                    ui.label(egui::RichText::new(format!("(ID: {})", item_id))
                                                                        .color(egui::Color32::from_rgb(150, 150, 255))
                                                                        .small());
                                                                    ui.end_row();
                                                                }
                                                            });
                                                    });
                                            });
                                    }

                                    if foreground_counts.is_empty() && background_counts.is_empty() {
                                        ui.label(egui::RichText::new("No tiles placed in world")
                                            .color(egui::Color32::from_rgb(150, 150, 150)));
                                    }
                                }
                            });

                    } else {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("Unable to access world data")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(255, 200, 100)));
                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("World data might be locked or not loaded yet.")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(150, 150, 150)));
                            ui.add_space(20.0);
                        });
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("No bot selected")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(255, 200, 100)));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Please select a bot to view world information.")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(150, 150, 150)));
                        ui.add_space(20.0);
                    });
                }
            });
        
        if close_world_info {
            ui_state.world_info_window_open = false;
        }
    }

    if ui_state.settings_window_open {
        egui::Window::new("Settings").show(contexts.ctx_mut()?, |ui| {
            ui.label("Settings will be here.");
        });
    }

    Ok(())
}