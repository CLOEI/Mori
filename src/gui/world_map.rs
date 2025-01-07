use crate::texture_manager::TextureManager;
use crate::utils::color;
use crate::{manager::bot_manager::BotManager, types::config::BotConfig, utils};
use eframe::egui::{self, Color32, Pos2, Rect, Ui};
use egui::{Painter};
use gtworld_r::TileType;
use paris::info;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;

#[derive(Default)]
pub struct WorldMap {
    pub selected_bot: String,
    pub warp_name: String,
    pub bots: Vec<BotConfig>,
}

impl WorldMap {
    pub fn render(
        &mut self,
        ui: &mut Ui,
        manager: &Arc<RwLock<BotManager>>,
        texture_manager: &Arc<RwLock<TextureManager>>,
    ) {
        self.bots = utils::config::get_bots();
        self.selected_bot = utils::config::get_selected_bot();

        if !self.selected_bot.is_empty() {
            let bot = {
                let manager = manager.read().unwrap();

                manager.get_bot(&self.selected_bot).cloned()
            };

            if let Some(bot) = bot {
                let available_size = ui.available_size();
                let world = bot.world.read().unwrap();

                let tile_width = available_size.x / world.width as f32;
                let tile_height = available_size.y / world.height as f32;

                let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::hover());
                let offset_x = rect.min.x;
                let offset_y = rect.min.y;
                let draw_list = ui.painter_at(rect);

                draw_list.rect_filled(rect, 0.0, Color32::from_rgb(96, 215, 255));

                for y in 0..world.height {
                    for x in 0..world.width {
                        let cell_min = Pos2::new(
                            offset_x + x as f32 * tile_width,
                            offset_y + y as f32 * tile_height,
                        );
                        let cell_max = Pos2::new(cell_min.x + tile_width, cell_min.y + tile_height);
                        let tile = match world.get_tile(x, y) {
                            Some(tile) => tile.clone(),
                            None => {
                                draw_list.rect_filled(
                                    Rect::from_min_max(cell_min, cell_max),
                                    0.0,
                                    Color32::from_rgb(255, 215, 0),
                                );
                                continue;
                            }
                        };

                        let (foreground, foreground_seed) = {
                            let item_database = bot.item_database.read().unwrap();
                            let foreground = item_database
                                .get_item(&(tile.foreground_item_id as u32))
                                .unwrap();
                            let foreground_seed = item_database
                                .get_item(&((tile.foreground_item_id + 1) as u32))
                                .unwrap();
                            (foreground, foreground_seed)
                        };
                        let (background, background_seed) = {
                            let item_database = bot.item_database.read().unwrap();
                            let background = item_database
                                .get_item(&(tile.background_item_id as u32))
                                .unwrap();
                            let background_seed = item_database
                                .get_item(&((tile.background_item_id + 1) as u32))
                                .unwrap();
                            (background, background_seed)
                        };

                        if tile.background_item_id != 0 || tile.foreground_item_id != 0 {
                            if tile.background_item_id != 0 {
                                self.draw_color(
                                    &draw_list,
                                    cell_min,
                                    cell_max,
                                    background_seed.base_color,
                                )
                            }
                            if tile.foreground_item_id != 0 {
                                if tile.foreground_item_id % 2 != 0 {
                                    self.draw_color(
                                        &draw_list,
                                        cell_min,
                                        cell_max,
                                        foreground.overlay_color,
                                    )
                                } else {
                                    self.draw_color(
                                        &draw_list,
                                        cell_min,
                                        cell_max,
                                        foreground_seed.base_color,
                                    )
                                }
                            }
                        }

                        for player in bot.players.lock().unwrap().clone() {
                            if (player.position.x / 32.0).floor() == (x as f32)
                                && (player.position.y / 32.0).floor() == (y as f32)
                            {
                                draw_list.rect_filled(
                                    Rect::from_min_max(cell_min, cell_max),
                                    0.0,
                                    Color32::from_rgb(255, 0, 0),
                                );
                            }
                        }

                        let (bot_name, bot_position) = {
                            let position = bot.position.lock().unwrap();
                            let info = bot.info.lock().unwrap();
                            (info.login_info.tank_id_name.clone(), position.clone())
                        };
                        if (bot_position.x / 32.0).floor() == (x as f32)
                            && (bot_position.y / 32.0).floor() == (y as f32)
                        {
                            draw_list.rect_filled(
                                Rect::from_min_max(cell_min, cell_max),
                                0.0,
                                Color32::from_rgb(255, 215, 0),
                            );
                        }

                        if response.hover_pos().map_or(false, |pos| {
                            Rect::from_min_max(cell_min, cell_max).contains(pos)
                        }) {
                            let data = if let TileType::Seed {
                                ready_to_harvest,
                                elapsed,
                                ..
                            } = &tile.tile_type
                            {
                                let elapsed_secs = elapsed.as_secs();
                                let ready = if *ready_to_harvest {
                                    "Yes"
                                } else {
                                    if world.is_tile_harvestable(&tile) {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                };
                                format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nReady to harvest: {}\nTime passed: {}\nRender type: {}",
                                    x, y, foreground.name, foreground.collision_type, ready, elapsed_secs, foreground.render_type
                                )
                            } else if let TileType::ChemicalSource {
                                ready_to_harvest,
                                elapsed,
                                time_passed,
                            } = &tile.tile_type
                            {
                                let elapsed_secs = Instant::now().elapsed().as_secs();
                                let ready = if *ready_to_harvest {
                                    "Yes"
                                } else {
                                    if world.is_tile_harvestable(&tile) {
                                        "Yes"
                                    } else {
                                        "No"
                                    }
                                };
                                format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nReady to harvest: {}\nTime passed: {}\nRender type: {}",
                                    x, y, foreground.name, foreground.collision_type, ready, elapsed_secs, foreground.render_type
                                )
                            } else {
                                format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nRender type: {}",
                                    x, y, foreground.name, foreground.collision_type, foreground.render_type
                                )
                            };

                            egui::show_tooltip(
                                ui.ctx(),
                                ui.layer_id(),
                                egui::Id::new("tile_info"),
                                |ui| {
                                    ui.label(egui::RichText::new(data).monospace());
                                },
                            );

                            if ui.input(|i| i.pointer.any_click()) {
                                info!("Clicked on tile: {}|{}", x, y);
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot_clone.find_path(x as u32, y as u32);
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn draw_color(&self, draw_list: &Painter, cell_min: Pos2, cell_max: Pos2, color: u32) {
        let (b, g, r, _) = color::extract_bgra(color);
        draw_list.rect_filled(
            Rect::from_min_max(cell_min, cell_max),
            0.0,
            Color32::from_rgb(r, g, b),
        );
    }
}
