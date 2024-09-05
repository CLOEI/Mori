use std::sync::{Arc, RwLock};
use std::thread;
use crate::{
    bot::{self},
    manager::Manager,
    types::config::BotConfig,
    utils,
};
use eframe::egui::{self, Color32, Pos2, Rect, Ui};
use paris::info;
use crate::bot::features;

#[derive(Default)]
pub struct WorldMap {
    pub selected_bot: String,
    pub warp_name: String,
    pub bots: Vec<BotConfig>,
}

impl WorldMap {
    pub fn render(&mut self, ui: &mut Ui, manager: &Arc<RwLock<Manager>>) {
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
                let draw_list = ui.painter();
                let p = ui.min_rect().min;
                let size = ui.available_size();
                let min = Pos2::new(p.x, p.y);
                let max = Pos2::new(p.x + size.x, p.y + size.y);
                let rect = Rect::from_min_max(min, max);
                draw_list.rect_filled(rect, 0.0, Color32::WHITE);

                let world = bot.world.read().unwrap();
                let cell_width = size.x / world.width as f32;
                let cell_height = size.y / world.height as f32;

                for y in 0..world.height {
                    for x in 0..world.width {
                        let cell_min = Pos2::new(
                            min.x + x as f32 * cell_width,
                            min.y + y as f32 * cell_height,
                        );
                        let cell_max = Pos2::new(cell_min.x + cell_width, cell_min.y + cell_height);
                        if (y * world.width + x) >= world.tile_count {
                            draw_list.rect_filled(
                                Rect::from_min_max(cell_min, cell_max),
                                0.0,
                                Color32::from_rgb(255, 215, 0),
                            );
                            continue;
                        }
                        let tile = world.get_tile(x, y).unwrap();
                        let item = bot
                            .item_database
                            .get_item(&(tile.foreground_item_id as u32))
                            .unwrap();

                        let color = bot
                            .item_database
                            .get_item(&((tile.foreground_item_id + 1) as u32))
                            .unwrap()
                            .overlay_color;

                        let r = ((color >> 24) & 0xFF) as u8;
                        let g = ((color >> 16) & 0xFF) as u8;
                        let b = ((color >> 8) & 0xFF) as u8;
                        let a = (color & 0xFF) as u8;

                        draw_list.rect_filled(
                            Rect::from_min_max(cell_min, cell_max),
                            0.0,
                            Color32::from_rgba_unmultiplied(r, g, b, a),
                        );

                        for player in bot.players.read().unwrap().clone() {
                            if player.position.x / 32.0 == (x as f32)
                                && player.position.y / 32.0 == (y as f32)
                            {
                                draw_list.rect_filled(
                                    Rect::from_min_max(cell_min, cell_max),
                                    0.0,
                                    Color32::from_rgb(255, 215, 0),
                                );
                            }
                        }

                        let bot_position = bot.position.read().unwrap();
                        if bot_position.x / 32.0 == (x as f32)
                            && bot_position.y / 32.0 == (y as f32)
                        {
                            draw_list.rect_filled(
                                Rect::from_min_max(cell_min, cell_max),
                                0.0,
                                Color32::from_rgb(255, 0, 0),
                            );
                        }

                        if ui.rect_contains_pointer(Rect::from_min_max(cell_min, cell_max)) {
                            egui::show_tooltip(
                                ui.ctx(),
                                ui.layer_id(),
                                egui::Id::new("tile_info"),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Position: {}|{}\nItem name: {}\nCollision type: {}",
                                            x, y, item.name, item.collision_type
                                        ))
                                            .monospace(),
                                    );
                                },
                            );
                            if ui.input(|i| i.pointer.any_click()) {
                                info!("Clicked on tile: {}|{}", x, y);
                                let bot_clone = bot.clone();
                                thread::spawn(move || {
                                    bot::find_path(&bot_clone, x, y);
                                });
                            }
                        }
                    }
                }

                egui::Window::new("Movement").show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Up").clicked() {
                            let bot_clone = bot.clone();
                            thread::spawn(move || {
                                bot::walk(&bot_clone, 0, -1, false);
                            });
                        }
                        if ui.button("Down").clicked() {
                            let bot_clone = bot.clone();
                            thread::spawn(move || {
                                bot::walk(&bot_clone, 0, 1, false);
                            });
                        }
                        if ui.button("Left").clicked() {
                            let bot_clone = bot.clone();
                            thread::spawn(move || {
                                bot::walk(&bot_clone, -1, 0, false);
                            });
                        }
                        if ui.button("Right").clicked() {
                            let bot_clone = bot.clone();
                            thread::spawn(move || {
                                bot::walk(&bot_clone, 1, 0, false);
                            });
                        }
                        if ui.button("Dev button").clicked() { // this button used only for dev purpose, change it to your needs
                            let bot_clone = bot.clone();
                            thread::spawn(move || {
                                features::auto_clear_world::start(&bot_clone);
                            });
                        }
                    });
                });
            }
        }
    }
}
