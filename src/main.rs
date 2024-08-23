use std::{sync::Arc, thread, time::Duration};

use bot::Bot;
use paris::info;
use types::elogin_method::ELoginMethod;

mod bot;
mod types;
mod utils;

fn main() {
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
    let bot = Arc::new(Bot::new(
        "".to_string(),
        "".to_string(),
        "recovery_code".to_string(),
        ELoginMethod::GOOGLE,
        Arc::new(item_database),
    ));

    let bot_clone = bot.clone();
    thread::spawn(move || bot::logon(&bot_clone));

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
