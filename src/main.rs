use std::{
    fs::{self, File},
    io::Write,
    sync::Arc,
    thread,
    time::Duration,
};

use bot::Bot;
use paris::info;
use types::config::Config;

mod bot;
mod types;
mod utils;

fn init_config() {
    if !fs::metadata("config.json").is_ok() {
        let mut file = File::create("config.json").unwrap();
        let config = Config {
            bots: Vec::new(),
            timeout: 5,
            findpath_delay: 5,
        };

        let j = serde_json::to_string_pretty(&config).unwrap();
        file.write_all(j.as_bytes()).unwrap();
    }
}

fn main() {
    init_config();
    let item_database = {
        match gtitem_r::load_from_file("items.dat") {
            Ok(item_database) => {
                info!("Item database loaded successfully");
                item_database
            }
            Err(e) => {
                info!("Failed to load item database: {}", e);
                return;
            }
        }
    };

    let bots = utils::config::get_bots();
    let item_database = Arc::new(item_database);
    for bot in bots {
        let bot_data = bot.data.clone();
        let bot_instance = Arc::new(Bot::new(bot, item_database.clone()));

        let bot_clone = bot_instance.clone();
        thread::spawn(move || bot::logon(&bot_clone, bot_data));
    }

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
