use gt_core::Socks5Config;
use gt_core::types::bot;
use gt_core::types::status::PeerStatus;
use gt_core::{Bot, gtitem_r};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

fn main() {
    let item_database = Arc::new(RwLock::new(gtitem_r::load_from_file("items.dat").unwrap()));
    println!(
        "Loaded {} items from items.dat",
        item_database.read().unwrap().item_count
    );

    let (bot, _) = Bot::new(
        gt_core::types::bot::LoginVia::LEGACY(["".into(), "".into()]),
        None,
        item_database.clone(),
None,
    );
    let bot_clone = bot.clone();
    thread::spawn(move || {
        bot_clone.logon(None);
    });

    while bot.peer_status() != PeerStatus::InWorld {};
    println!("Bot is now in world!");
    bot.say("Hmm");
    thread::sleep(Duration::from_secs(5));

    fn place(bot: &Bot) {
        let offsets = [(-2, -1), (-1, -1), (0, -1), (1, -1), (2, -1)];
        for (offset_x, offset_y) in offsets.iter() {
            if check_tile(bot, (*offset_x, *offset_y)) {
                bot.place(*offset_x, *offset_y, 8640, false);
            }
        }
    }

    fn check_tile(bot: &Bot, offset: (i32, i32)) -> bool {
        let pos = bot.movement.position();
        let target_x = pos.0 as i32 + offset.0 * 32;
        let target_y = pos.1 as i32 + offset.1 * 32;
        let binding = bot.world.data.lock().unwrap();
        let tile = binding.get_tile(target_x as u32 / 32, target_y as u32 / 32);
        if let Some(tile) = tile {
            tile.foreground_item_id == 0
        } else {
            false
        }
    }

    'main_loop: loop {
        if bot.peer_status() != PeerStatus::InWorld {
            continue 'main_loop;
        }

        if bot.inventory.get_item_count(8640) == 0 {
            println!("No more items (8640) in inventory. Exiting...");
            break 'main_loop;
        }

        let current_pos = bot.movement.position();
        let target_tile = (19, 52);
        let current_tile = ((current_pos.0 / 32.0) as u32, (current_pos.1 / 32.0) as u32);

        if current_tile != target_tile {
            bot.find_path(target_tile.0, target_tile.1);
        }

        if bot.peer_status() != PeerStatus::InWorld {
            continue 'main_loop;
        }
        place(&bot);
    }
}
