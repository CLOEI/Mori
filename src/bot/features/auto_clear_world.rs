use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::bot;
use crate::bot::{find_path, punch, Bot};

static CAVE_BACKGROUND: u16 = 14;
static BEDROCK: u16 = 8;

pub fn start(bot: &Arc<Bot>) {
    let (world_width, world_height) = {
        let world = bot.world.read().unwrap();
        (world.width, world.height)
    };

    for y in 23..world_height - 6 {
        for x in 0..world_width {
            while bot::is_inworld(&bot) {
                // Log the current coordinates being processed
                println!("Processing x: {}, y: {}", x, y);

                let (foreground_id, background_id) = {
                    let world = bot.world.read().unwrap();
                    if let Some(tile) = world.get_tile(x, y) {
                        (tile.foreground_item_id, tile.background_item_id)
                    } else {
                        break;
                    }
                };

                println!(
                    "At x: {}, y: {}, foreground_id: {}, background_id: {}",
                    x, y, foreground_id, background_id
                );

                if background_id != CAVE_BACKGROUND || foreground_id == BEDROCK {
                    println!("Breaking at x: {}, y: {} due to tile conditions", x, y);
                    break;
                }

                println!("Calling find_path for x: {}, y: {}", x, y - 1);
                find_path(&bot, x, y - 1);

                println!("Sleeping for a bit at x: {}, y: {}", x, y);

                println!("Punching at x: {}, y: {}", x, y);
                punch(&bot, 0, 1);

                println!("Completed loop for x: {}, y: {}", x, y);
            }
        }
    }
}
