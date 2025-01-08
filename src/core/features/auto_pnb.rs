use crate::manager::bot_manager::BotManager;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct AutoPNB {
    pub selected_bot: String,
    pub is_farming: bool,
    pub is_paused: Arc<AtomicBool>,
    pub selected_positions: Vec<(i32, i32)>,
    pub item_id: u32,
    pub grid_selection: [[bool; 5]; 5],
    pub temp_item_id: String,
}

impl AutoPNB {
    pub fn start_autofarm(&mut self, manager: Arc<RwLock<BotManager>>) {
        if self.selected_positions.is_empty() || self.is_farming {
            return;
        }
        if self.item_id == 0 {
            return;
        }
        self.is_farming = true;
        self.is_paused.store(false, Ordering::SeqCst);
        let positions = self.selected_positions.clone();
        let item_id = self.item_id;
        let bot_name = self.selected_bot.clone();
        let is_paused = Arc::clone(&self.is_paused);

        thread::spawn(move || {
            if let Some(bot) = manager.read().unwrap().get_bot(&bot_name).cloned() {
                for (offset_x, offset_y) in positions.iter().cycle() {
                    if is_paused.load(Ordering::SeqCst) {
                        println!("PNB paused.");
                        return;
                    }

                    let tiles = {
                        let world = bot.world.try_read().unwrap();
                        world.tiles.clone()
                    };

                    let local = {
                        let position = bot.position.lock().unwrap();
                        position.clone()
                    };

                    let target_x = (local.x / 32.0) as i32 + offset_x;
                    let target_y = (local.y / 32.0) as i32 + offset_y;

                    for tile in tiles {
                        if tile.x as i32 == target_x && tile.y as i32 == target_y {
                            if tile.foreground_item_id as u32 != item_id {
                                println!(
                                    "Placing item at ({}, {}). Target item_id: {}, Current: {}",
                                    target_x, target_y, item_id, tile.foreground_item_id
                                );
                                thread::sleep(Duration::from_millis(100));
                                bot.place(*offset_x, *offset_y, item_id);
                                thread::sleep(Duration::from_millis(100));
                            } else if tile.foreground_item_id as u32 == item_id || tile.foreground_item_id != 0 {
                                println!(
                                    "Punching tile at ({}, {}). Current item_id: {}",
                                    target_x, target_y, tile.foreground_item_id
                                );
                                thread::sleep(Duration::from_millis(170));
                                bot.punch(*offset_x, *offset_y);
                            }
                        }
                    }
                }
            } else {
                println!("Bot '{}' not found.", bot_name);
            }
        });
    }

    pub fn pause_autofarm(&mut self) {
        println!("Pausing autofarm...");
        self.is_paused.store(true, Ordering::SeqCst);
        self.is_farming = false;
    }
}
