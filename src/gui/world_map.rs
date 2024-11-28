use crate::core::Bot;
use crate::texture_manager::TextureManager;
use crate::utils::color;
use crate::{manager::bot_manager::BotManager, types::config::BotConfig, utils};
use eframe::egui::{self, Color32, Pos2, Rect, Ui};
use egui::Painter;
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

                match manager.get_bot(&self.selected_bot) {
                    Some(bot) => Some(bot.clone()),
                    None => None,
                }
            };
            if let Some(bot) = bot {
                let size = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
                let draw_list = ui.painter_at(rect);

                draw_list.rect_filled(rect, 0.0, Color32::from_rgb(96, 215, 255));

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

                        if world_x <= 0 || world_y <= 0 || world_x >= world.width as i32 || world_y >= world.height as i32 {
                            continue;
                        }

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
                                self.draw_texture(
                                    &draw_list,
                                    texture_manager,
                                    background.texture_x,
                                    background.texture_y,
                                    background.texture_file_name,
                                    cell_min,
                                    cell_max,
                                    tile.flags.flipped_x,
                                    Color32::WHITE,
                                    background_seed.base_color,
                                )
                            }
                            if tile.foreground_item_id != 0 {
                                if tile.foreground_item_id % 2 != 0 {
                                    let (b, g, r, a) =
                                        utils::color::extract_bgra(foreground.overlay_color);
                                    let (spread_x, spread_y) = match foreground.render_type {
                                        2 | 5 => (4.0, 1.0),
                                        4 => (4.0, 0.0),
                                        3 | 7 | 8 | 9 | 10 => (3.0, 0.0),
                                        _ => (0.0, 0.0),
                                    };

                                    self.draw_texture(
                                        &draw_list,
                                        texture_manager,
                                        foreground.tree_base_sprite,
                                        19,
                                        "tiles_page1.rttex".to_string(),
                                        cell_min,
                                        cell_max,
                                        tile.flags.flipped_x,
                                        Color32::WHITE,
                                        foreground.base_color,
                                    );
                                    self.draw_texture(
                                        &draw_list,
                                        texture_manager,
                                        foreground.tree_overlay_sprite,
                                        18,
                                        "tiles_page1.rttex".to_string(),
                                        cell_min,
                                        cell_max,
                                        tile.flags.flipped_x,
                                        Color32::from_rgba_unmultiplied(r, g, b, a),
                                        foreground.overlay_color,
                                    );
                                    let new_cell_min = Pos2::new(
                                        cell_min.x + cell_size * 0.375,
                                        cell_min.y + cell_size * 0.375,
                                    );
                                    let new_cell_max = Pos2::new(
                                        cell_max.x - cell_size * 0.375,
                                        cell_max.y - cell_size * 0.375,
                                    );

                                    self.draw_texture(
                                        &draw_list,
                                        texture_manager,
                                        foreground.texture_x + spread_x as u8,
                                        foreground.texture_y + spread_y as u8,
                                        foreground.texture_file_name,
                                        new_cell_min,
                                        new_cell_max,
                                        tile.flags.flipped_x,
                                        Color32::WHITE,
                                        foreground.base_color,
                                    );

                                    draw_list.rect_stroke(
                                        Rect::from_min_max(new_cell_min, new_cell_max),
                                        0.2,
                                        (2.0, Color32::WHITE),
                                    );
                                } else {
                                    let flipped =
                                        if foreground.flags.flippable && tile.flags.flipped_x {
                                            true
                                        } else {
                                            false
                                        };
                                    let activated =
                                        if tile.flags.is_on && foreground.render_type == 1 {
                                            true
                                        } else {
                                            false
                                        };
                                    let (mut texture_x, mut texture_y) = (
                                        foreground.texture_x,
                                        foreground.texture_y,
                                    );

                                    let top_left_tile = world.get_tile(world_x as u32 - 1, world_y as u32 - 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let top_center_tile = world.get_tile(world_x as u32, world_y as u32 - 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let top_right_tile = world.get_tile(world_x as u32 + 1, world_y as u32 - 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let center_left_tile = world.get_tile(world_x as u32 - 1, world_y as u32).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let center_right_tile = world.get_tile(world_x as u32 + 1, world_y as u32).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let bottom_left_tile = world.get_tile(world_x as u32 - 1, world_y as u32 + 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let bottom_center_tile = world.get_tile(world_x as u32, world_y as u32 + 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let bottom_right_tile = world.get_tile(world_x as u32 + 1, world_y as u32 + 1).map_or(None, |tile| {
                                        if tile.foreground_item_id != foreground.id as u16 {
                                            None
                                        } else {
                                            Some(tile)
                                        }
                                    });
                                    let top_center_item = world.get_tile(world_x as u32, world_y as u32 - 1).map_or(None, |tile| {
                                        Some(tile)
                                    });

                                    if foreground.render_type == 2 {
                                        self.add_render_type2(
                                            &mut texture_x,
                                            &mut texture_y,
                                            top_center_tile,
                                            center_left_tile,
                                            center_right_tile,
                                            bottom_center_tile,
                                        );
                                    } else if foreground.render_type == 7 {
                                        self.add_render_type7(
                                            &mut texture_x,
                                            top_center_tile,
                                            bottom_center_tile,
                                        )
                                    } else if foreground.render_type == 3 {
                                        self.add_render_type3(
                                            &mut texture_x,
                                            top_center_tile,
                                            center_left_tile,
                                            center_right_tile,
                                            bottom_center_tile,
                                            top_center_item
                                        )
                                    }

                                    if let TileType::DisplayBlock { item_id } = tile.tile_type {
                                        self.draw_display_block(
                                            &draw_list,
                                            &bot,
                                            &texture_manager,
                                            item_id,
                                            cell_min,
                                            cell_max,
                                        );
                                    }

                                    self.draw_texture(
                                        &draw_list,
                                        texture_manager,
                                        if activated {
                                            texture_x + 1
                                        } else {
                                            texture_x
                                        },
                                        texture_y,
                                        foreground.texture_file_name,
                                        cell_min,
                                        cell_max,
                                        flipped,
                                        Color32::WHITE,
                                        foreground_seed.base_color,
                                    );

                                    if let TileType::VendingMachine { item_id, .. } = tile.tile_type
                                    {
                                        self.draw_vending_machine(
                                            &draw_list,
                                            &bot,
                                            &texture_manager,
                                            item_id,
                                            cell_min,
                                            cell_max,
                                        );
                                    }
                                    if let TileType::Shelf {
                                        top_left_item_id,
                                        top_right_item_id,
                                        bottom_left_item_id,
                                        bottom_right_item_id,
                                    } = tile.tile_type
                                    {
                                        self.draw_display_shelf(
                                            &draw_list,
                                            &bot,
                                            &texture_manager,
                                            top_left_item_id,
                                            top_right_item_id,
                                            bottom_left_item_id,
                                            bottom_right_item_id,
                                            cell_min,
                                            cell_max,
                                        );
                                    }
                                }
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
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nReady to harvest: {}\nTime passed: {}\nRender type: {}",
                                    world_x, world_y, foreground.name, foreground.collision_type, ready_to_harvest, elapsed, foreground.render_type
                                )
                            } else {
                                data = format!(
                                    "Position: {}|{}\nItem name: {}\nCollision type: {}\nRender type: {}",
                                    world_x, world_y, foreground.name, foreground.collision_type, foreground.render_type
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

    fn add_render_type3(
        &self,
        texture_x: &mut u8,
        top_center_tile: Option<&gtworld_r::Tile>,
        center_left_tile: Option<&gtworld_r::Tile>,
        center_right_tile: Option<&gtworld_r::Tile>,
        bottom_center_tile: Option<&gtworld_r::Tile>,
        top_center_item: Option<&gtworld_r::Tile>,
    ) {
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_none()  && bottom_center_tile.is_none() {
            *texture_x += 3;
        }
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_some()  && bottom_center_tile.is_none() {
            match top_center_item {
                Some(tile) => {
                    if tile.foreground_item_id == 8986 {
                        *texture_x += 4;
                    } else {
                        *texture_x += 0;
                    }
                }
                None => {
                    *texture_x += 0;
                }
            }
        }
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_some()  && bottom_center_tile.is_some() {
            match top_center_item {
                Some(tile) => {
                    if tile.foreground_item_id == 8986 {
                        *texture_x += 4;
                    } else {
                        *texture_x += 0;
                    }
                }
                None => {
                    *texture_x += 0;
                }
            }
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_none()  && bottom_center_tile.is_none() {
            match top_center_item {
                Some(tile) => {
                    if tile.foreground_item_id == 8986 {
                        *texture_x += 5;
                    } else {
                        *texture_x += 0;
                    }
                }
                None => {
                    *texture_x += 0;
                }
            }
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_none()  && bottom_center_tile.is_some() {
            match top_center_item {
                Some(tile) => {
                    if tile.foreground_item_id == 8986 {
                        *texture_x += 5;
                    } else {
                        *texture_x += 0;
                    }
                }
                None => {
                    *texture_x += 0;
                }
            }
        }
        if top_center_tile.is_some() && center_left_tile.is_some() && center_right_tile.is_some()  && bottom_center_tile.is_none() {
            *texture_x += 1;
        }
        if top_center_tile.is_some() && center_left_tile.is_some() && center_right_tile.is_some()  && bottom_center_tile.is_some() {
            *texture_x += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_some()  && bottom_center_tile.is_none() {
            *texture_x += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_some()  && bottom_center_tile.is_some() {
            *texture_x += 1;
        }
    }

    fn add_render_type7(
        &self,
        texture_x: &mut u8,
        top_center_tile: Option<&gtworld_r::Tile>,
        bottom_center_tile: Option<&gtworld_r::Tile>,
    ) {
        if top_center_tile.is_none() && bottom_center_tile.is_none() {
            *texture_x += 3;
        }
        if top_center_tile.is_none() && bottom_center_tile.is_some() {
            *texture_x += 2;
        }
        if top_center_tile.is_some() && bottom_center_tile.is_some() {
            *texture_x += 1;
        }
    }

    fn add_render_type2(
        &self,
        texture_x: &mut u8,
        texture_y: &mut u8,
        top_center_tile: Option<&gtworld_r::Tile>,
        center_left_tile: Option<&gtworld_r::Tile>,
        center_right_tile: Option<&gtworld_r::Tile>,
        bottom_center_tile: Option<&gtworld_r::Tile>,
    ) {
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_none()  && bottom_center_tile.is_none() {
            *texture_x += 4;
            *texture_y += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_some() && bottom_center_tile.is_none() {
            *texture_x += 5;
            *texture_y += 3;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_none() && bottom_center_tile.is_none() {
            *texture_x += 6;
            *texture_y += 3;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_some() && bottom_center_tile.is_none() {
            *texture_x += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_none() && bottom_center_tile.is_some() {
            *texture_x += 2;
            *texture_y += 1;
        }
        if top_center_tile.is_some() && center_left_tile.is_none() && center_right_tile.is_none() && bottom_center_tile.is_none() {
            *texture_x += 3;
            *texture_y += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_some() && bottom_center_tile.is_some() {
            *texture_x += 1;
        }
        if top_center_tile.is_none() && center_left_tile.is_some() && center_right_tile.is_none() && bottom_center_tile.is_some() {
            *texture_x += 6;
        }
        if top_center_tile.is_none() && center_left_tile.is_none() && center_right_tile.is_some() && bottom_center_tile.is_some() {
            *texture_x += 5;
        }
        if top_center_tile.is_some() && center_left_tile.is_some() && center_right_tile.is_none() && bottom_center_tile.is_some() {
            *texture_x += 4;
        }
        if top_center_tile.is_some() && center_left_tile.is_none() && center_right_tile.is_some() && bottom_center_tile.is_some() {
            *texture_x += 3;
        }
        if top_center_tile.is_some() && center_left_tile.is_none() && center_right_tile.is_some() && bottom_center_tile.is_none() {
            *texture_x += 7;
        }
        if top_center_tile.is_some() && center_left_tile.is_some() && center_right_tile.is_none() && bottom_center_tile.is_none() {
            *texture_y += 1;
        }
    }

    fn draw_display_shelf(
        &self,
        draw_list: &Painter,
        bot: &Arc<Bot>,
        texture_manager: &Arc<RwLock<TextureManager>>,
        top_left_item_id: u32,
        top_right_item_id: u32,
        bottom_left_item_id: u32,
        bottom_right_item_id: u32,
        cell_min: Pos2,
        cell_max: Pos2,
    ) {
        let (top_left_item, top_left_item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let top_left_item = item_database.get_item(&top_left_item_id).unwrap();
            let top_left_item_seed = item_database.get_item(&(top_left_item_id + 1)).unwrap();
            (top_left_item, top_left_item_seed)
        };

        let (top_right_item, top_right_item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let top_right_item = item_database.get_item(&top_right_item_id).unwrap();
            let top_right_item_seed = item_database.get_item(&(top_right_item_id + 1)).unwrap();
            (top_right_item, top_right_item_seed)
        };

        let (bottom_left_item, bottom_left_item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let bottom_left_item = item_database.get_item(&bottom_left_item_id).unwrap();
            let bottom_left_item_seed = item_database.get_item(&(bottom_left_item_id + 1)).unwrap();
            (bottom_left_item, bottom_left_item_seed)
        };

        let (bottom_right_item, bottom_right_item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let bottom_right_item = item_database.get_item(&bottom_right_item_id).unwrap();
            let bottom_right_item_seed = item_database.get_item(&(bottom_right_item_id + 1)).unwrap();
            (bottom_right_item, bottom_right_item_seed)
        };

        let half_width = (cell_max.x - cell_min.x) / 2.0;
        let half_height = (cell_max.y - cell_min.y) / 2.0;

        let top_left_min = cell_min;
        let top_left_max = Pos2::new(cell_min.x + half_width, cell_min.y + half_height);

        let top_right_min = Pos2::new(cell_min.x + half_width, cell_min.y);
        let top_right_max = Pos2::new(cell_max.x, cell_min.y + half_height);

        let bottom_left_min = Pos2::new(cell_min.x, cell_min.y + half_height);
        let bottom_left_max = Pos2::new(cell_min.x + half_width, cell_max.y);

        let bottom_right_min = Pos2::new(cell_min.x + half_width, cell_min.y + half_height);
        let bottom_right_max = cell_max;

        if top_left_item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                top_left_item.texture_x,
                top_left_item.texture_y,
                top_left_item.texture_file_name,
                top_right_min,
                top_right_max,
                false,
                Color32::WHITE,
                top_left_item_seed.base_color,
            )
        }

        if top_right_item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                top_right_item.texture_x,
                top_right_item.texture_y,
                top_right_item.texture_file_name,
                top_left_min,
                top_left_max,
                false,
                Color32::WHITE,
                top_right_item_seed.base_color,
            );
        }

        if bottom_left_item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                bottom_left_item.texture_x,
                bottom_left_item.texture_y,
                bottom_left_item.texture_file_name,
                bottom_left_min,
                bottom_left_max,
                false,
                Color32::WHITE,
                bottom_left_item_seed.base_color,
            );
        }

        if bottom_right_item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                bottom_right_item.texture_x,
                bottom_right_item.texture_y,
                bottom_right_item.texture_file_name,
                bottom_right_min,
                bottom_right_max,
                false,
                Color32::WHITE,
                bottom_right_item_seed.base_color,
            );
        }
    }

    fn draw_vending_machine(
        &self,
        draw_list: &Painter,
        bot: &Arc<Bot>,
        texture_manager: &Arc<RwLock<TextureManager>>,
        item_id: u32,
        cell_min: Pos2,
        cell_max: Pos2,
    ) {
        let (item, item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let item = item_database.get_item(&item_id).unwrap();
            let item_seed = item_database.get_item(&(item_id + 1)).unwrap();
            (item, item_seed)
        };

        let new_cell_min = Pos2::new(
            cell_min.x + (cell_max.x - cell_min.x) * 0.25,
            cell_min.y + (cell_max.y - cell_min.y) * 0.25,
        );
        let new_cell_max = Pos2::new(
            cell_max.x - (cell_max.x - cell_min.x) * 0.25,
            cell_max.y - (cell_max.y - cell_min.y) * 0.25,
        );

        if item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                item.texture_x,
                item.texture_y,
                item.texture_file_name,
                new_cell_min,
                new_cell_max,
                false,
                Color32::WHITE,
                item_seed.base_color,
            )
        }
    }

    fn draw_display_block(
        &self,
        draw_list: &Painter,
        bot: &Arc<Bot>,
        texture_manager: &Arc<RwLock<TextureManager>>,
        item_id: u32,
        cell_min: Pos2,
        cell_max: Pos2,
    ) {
        let (item, item_seed) = {
            let item_database = bot.item_database.read().unwrap();
            let item = item_database.get_item(&item_id).unwrap();
            let item_seed = item_database.get_item(&(item_id + 1)).unwrap();
            (item, item_seed)
        };

        if item.id != 0 {
            self.draw_texture(
                &draw_list,
                texture_manager,
                item.texture_x,
                item.texture_y,
                item.texture_file_name,
                cell_min,
                cell_max,
                false,
                Color32::WHITE,
                item_seed.base_color,
            )
        }
    }

    fn draw_texture(
        &self,
        draw_list: &Painter,
        texture_manager: &Arc<RwLock<TextureManager>>,
        texture_x: u8,
        texture_y: u8,
        texture_name: String,
        cell_min: Pos2,
        cell_max: Pos2,
        flipped: bool,
        color: Color32,
        base_color: u32,
    ) {
        match texture_manager.read().unwrap().get_texture(&texture_name) {
            Some(texture) => {
                let [width, height] = texture.size();
                let uv_x_start = (texture_x as f32 * 32.0) / width as f32;
                let uv_y_start = (texture_y as f32 * 32.0) / height as f32;
                let uv_x_end = ((texture_x as f32 * 32.0) + 32.0) / width as f32;
                let uv_y_end = ((texture_y as f32 * 32.0) + 32.0) / height as f32;

                let (uv_start, uv_end) = if flipped {
                    (
                        egui::Pos2::new(uv_x_end, uv_y_start),
                        egui::Pos2::new(uv_x_start, uv_y_end),
                    )
                } else {
                    (
                        egui::Pos2::new(uv_x_start, uv_y_start),
                        egui::Pos2::new(uv_x_end, uv_y_end),
                    )
                };

                let cell_min = Pos2::new(cell_min.x.round(), cell_min.y.round());
                let cell_max = Pos2::new(cell_max.x.round(), cell_max.y.round());

                draw_list.image(
                    texture.id(),
                    Rect::from_min_max(
                        Pos2::new(cell_min.x, cell_min.y),
                        Pos2::new(cell_max.x, cell_max.y),
                    ),
                    egui::Rect::from_min_max(uv_start, uv_end),
                    color,
                );
            }
            None => {
                let (b, g, r, _) = color::extract_bgra(base_color);
                draw_list.rect_filled(
                    Rect::from_min_max(cell_min, cell_max),
                    0.0,
                    Color32::from_rgb(r, g, b),
                );
            }
        }
    }
}
