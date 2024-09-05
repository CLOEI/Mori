use std::sync::Arc;
use crate::bot;
use crate::bot::{find_path, punch, Bot};

static CAVE_BACKGROUND: u16 = 14;
static BEDROCK: u16 = 8;

pub fn start(bot: &Arc<Bot>) {
    for y in 0..55 {
        for x in 0..100 {
            let is_inworld = bot::is_inworld(&bot);
            while is_inworld {
                let (foreground_id, background_id) = {
                    let world = bot.world.read();
                    if let Some(tile) = world.get_tile(x, y) {
                        (tile.foreground_item_id, tile.background_item_id)
                    } else {
                        break;
                    }
                };
                if background_id != CAVE_BACKGROUND {
                    break;
                }
                if foreground_id == BEDROCK {
                    break;
                }
                find_path(&bot, x, (y - 1));
                std::thread::sleep(std::time::Duration::from_millis(200));
                punch(&bot, 0, 1);
            }
        }
    }
}