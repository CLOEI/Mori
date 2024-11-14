use crate::core::features;
use crate::{
    core::{self},
    manager::bot_manager::BotManager,
    types::config::BotConfig,
    utils,
};
use eframe::egui::{self, Color32, Pos2, Rect, Ui};
use gtworld_r::TileType;
use paris::info;
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Default)]
pub struct WorldMap {
    pub selected_bot: String,
    pub warp_name: String,
    pub bots: Vec<BotConfig>,
    camera_pos: Pos2,
    zoom: f32,
}

impl WorldMap {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<BotManager>>) {
        self.bots = utils::config::get_bots();
        self.selected_bot = utils::config::get_selected_bot();

        if !self.selected_bot.is_empty() {
            let bot = {
                let manager = manager.read().unwrap();

                match manager.get_bot(&self.selected_bot) {
                    Some(bot) => Some(bot.clone()),
                    None => None,
                }
            };
            if let Some(bot) = bot {
                let size = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
                let draw_list = ui.painter_at(rect);

                if self.camera_pos == Pos2::default() {
                    let bot_position = bot.position.lock().unwrap();
                    self.camera_pos = Pos2::new(bot_position.x, bot_position.y);
                    self.zoom = 0.5;
                }

                {
                    let bot_position = bot.position.lock().unwrap();
                    let target_pos = Pos2::new(bot_position.x, bot_position.y);
                    let smoothing_factor = 0.1;
                    self.camera_pos.x += (target_pos.x - self.camera_pos.x) * smoothing_factor;
                    self.camera_pos.y += (target_pos.y - self.camera_pos.y) * smoothing_factor;
                }

                let cell_size = 32.0 * self.zoom;
                let camera_tile_x = (self.camera_pos.x / 32.0).floor() as i32;
                let camera_tile_y = (self.camera_pos.y / 32.0).floor() as i32;
                let offset_x = (self.camera_pos.x % 32.0) * self.zoom;
                let offset_y = (self.camera_pos.y % 32.0) * self.zoom;

                let tiles_in_view_x = (size.x / cell_size).ceil() as i32 + 1;
                let tiles_in_view_y = (size.y / cell_size).ceil() as i32 + 1;

                let world = bot.world.read().unwrap();
                for y in 0..tiles_in_view_y {
                    for x in 0..tiles_in_view_x {
                        let world_x = camera_tile_x + x - tiles_in_view_x / 2;
                        let world_y = camera_tile_y + y - tiles_in_view_y / 2;

                        let cell_min = Pos2::new(
                            rect.min.x + x as f32 * cell_size - offset_x,
                            rect.min.y + y as f32 * cell_size - offset_y,
                        );
                        let cell_max = Pos2::new(cell_min.x + cell_size, cell_min.y + cell_size);

                        if world_x < 0
                            || world_y < 0
                            || world_x >= world.width as i32
                            || world_y >= world.height as i32
                        {
                            continue;
                        }

                        if (world_y * world.width as i32 + world_x) >= world.tile_count as i32 {
                            draw_list.rect_filled(
                                Rect::from_min_max(cell_min, cell_max),
                                0.0,
                                Color32::from_rgb(255, 215, 0),
                            );
                            continue;
                        }
                        let tile = world.get_tile(world_x as u32, world_y as u32).unwrap();
                        let item = {
                            let item_database = bot.item_database.read().unwrap();
                            item_database
                                .get_item(&(tile.foreground_item_id as u32))
                                .unwrap()
                        };

                        if item.id == 0 {
                            draw_list.rect_filled(
                                Rect::from_min_max(
                                    Pos2::new(cell_min.x - 1.0, cell_min.y - 1.0),
                                    Pos2::new(cell_max.x + 1.0, cell_max.y + 1.0),
                                ),
                                0.0,
                                Color32::from_rgb(96, 215, 255),
                            );
                            if tile.background_item_id != 0 {
                                let item_database = bot.item_database.read().unwrap();
                                let item = item_database
                                    .get_item(&((tile.background_item_id + 1) as u32))
                                    .unwrap();
                                let color = item.base_color;
                                let r = ((color >> 24) & 0xFF) as u8;
                                let g = ((color >> 16) & 0xFF) as u8;
                                let b = ((color >> 8) & 0xFF) as u8;
                                draw_list.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(cell_min.x - 1.0, cell_min.y - 1.0),
                                        Pos2::new(cell_max.x + 1.0, cell_max.y + 1.0),
                                    ),
                                    0.0,
                                    Color32::from_rgb(b, g, r),
                                );
                            }
                        } else {
                            let items_database = bot.item_database.read().unwrap();
                            let color = items_database
                                .get_item(&((tile.foreground_item_id + 1) as u32))
                                .unwrap()
                                .base_color;
                            let r = ((color >> 24) & 0xFF) as u8;
                            let g = ((color >> 16) & 0xFF) as u8;
                            let b = ((color >> 8) & 0xFF) as u8;
                            if item.name == "Bedrock" {
                                draw_list.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(cell_min.x - 1.0, cell_min.y - 1.0),
                                        Pos2::new(cell_max.x + 1.0, cell_max.y + 1.0),
                                    ),
                                    0.0,
                                    Color32::from_rgb(105, 105, 105),
                                );
                            } else {
                                draw_list.rect_filled(
                                    Rect::from_min_max(
                                        Pos2::new(cell_min.x - 1.0, cell_min.y - 1.0),
                                        Pos2::new(cell_max.x + 1.0, cell_max.y + 1.0),
                                    ),
                                    0.0,
                                    Color32::from_rgb(b, g, r),
                                );
                            }
                        }

                        for player in bot.players.lock().unwrap().clone() {
                            if (player.position.x / 32.0).floor() == (world_x as f32)
                                && (player.position.y / 32.0).floor() == (world_y as f32)
                            {
                                draw_list.rect_filled(
                                    Rect::from_min_max(cell_min, cell_max),
                                    0.0,
                                    Color32::from_rgb(255, 215, 0),
                                );
                            }
                        }

                        let bot_position = bot.position.lock().unwrap();
                        if (bot_position.x / 32.0).floor() == (world_x as f32)
                            && (bot_position.y / 32.0).floor() == (world_y as f32)
                        {
                            draw_list.rect_filled(
                                Rect::from_min_max(cell_min, cell_max),
                                0.0,
                                Color32::from_rgb(255, 0, 0),
                            );
                        }

                        if response.hover_pos().map_or(false, |pos| {
                            Rect::from_min_max(cell_min, cell_max).contains(pos)
                        }) {
                            let data;
                            if let TileType::Seed {
                                ready_to_harvest,
                                timer,
                                ..
                            } = &tile.tile_type
                            {
                                let elapsed = timer.elapsed().as_secs();
                                let ready_to_harvest = if *ready_to_harvest {
                                    "Yes"
                                } else {
                                    if world.is_tile_harvestable(tile) {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                };
                                data = format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nReady to harvest: {}\nTime passed: {}",
                                    world_x, world_y, item.name, item.collision_type, ready_to_harvest, elapsed
                                )
                            } else {
                                data = format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}",
                                    world_x, world_y, item.name, item.collision_type
                                )
                            }

                            egui::show_tooltip(
                                ui.ctx(),
                                ui.layer_id(),
                                egui::Id::new("tile_info"),
                                |ui| {
                                    ui.label(egui::RichText::new(data).monospace());
                                },
                            );

                            if ui.input(|i| i.pointer.any_click()) {
                                info!("Clicked on tile: {}|{}", world_x, world_y);
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.find_path(world_x as u32, world_y as u32);
                                });
                            }
                        }
                    }
                }

                egui::Window::new("Movement")
                    .anchor(egui::Align2::RIGHT_BOTTOM, [0.0, 0.0])
                    .default_open(false)
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Up").clicked() {
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.walk(0, -1, false);
                                });
                            }
                            if ui.button("Down").clicked() {
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.walk(0, 1, false);
                                });
                            }
                            if ui.button("Left").clicked() {
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.walk(-1, 0, false);
                                });
                            }
                            if ui.button("Right").clicked() {
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.walk(1, 0, false);
                                });
                            }
                            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=2.0).text("Zoom"));
                        });
                    });

                egui::Window::new("FTUE")
                    .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
                    .default_open(false)
                    .show(ui.ctx(), |ui| {
                        ui.vertical(|ui| {
                            let ftue = {
                                let ftue = bot.ftue.lock().unwrap();
                                ftue.clone()
                            };

                            ui.label(format!("FTUE: {}", ftue.info));
                            ui.label(format!(
                                "Current progress: {}/{}",
                                ftue.current_progress, ftue.total_progress
                            ));
                        });
                    });
            }
        }
    }
}
