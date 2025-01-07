use std::sync::Arc;
use std::thread;
use crate::core::Bot;

pub fn start(bot: &Arc<Bot>, id: u16) {
    let bot = bot.clone();
    thread::spawn(move || {
        let tiles = {
            let world = bot.world.read().unwrap();
            world.tiles.iter().filter(|tile| {
                tile.foreground_item_id == id && world.is_tile_harvestable(tile)
            }).cloned().collect::<Vec<_>>()
        };

        for tile in tiles {
            bot.find_path(tile.x, tile.y);
            bot.punch(0, 0);
            thread::sleep(std::time::Duration::from_millis(350));

            loop {
                let world_name = {
                    let world = bot.world.read().unwrap();
                    world.name.clone()
                };
                let info = bot.info.lock().unwrap().status.clone();
                if info != "Disconnected" && world_name != "EXIT" {
                    break;
                }
                bot.log_warn("Currently disconnected, waiting for reconnection...");
                thread::sleep(std::time::Duration::from_secs(5));
            }
        }
        bot.log_info("Finish harvesting provider.")
    });
}