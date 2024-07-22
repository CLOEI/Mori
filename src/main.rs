mod bot;
mod manager;
mod types;
mod utils;
mod webserver;

use std::{fs, sync::Arc};

use manager::Manager;
use spdlog::prelude::*;

fn main() {
    let mut manager = match Manager::new() {
        Ok(manager) => manager,
        Err(err) => {
            error!("Error: {}", err);
            return;
        }
    };

    let data = match fs::read_to_string("data.json") {
        Ok(data) => data,
        Err(_) => {
            let data = webserver::Data {
                game_version: "4.61".to_string(),
                protocol: "209".to_string(),
                bots: vec![],
            };
            let serialized = serde_json::to_string_pretty(&data).expect("Failed to serialize data");
            fs::write("data.json", &serialized).unwrap();
            serialized
        }
    };
    let json = serde_json::from_str::<webserver::Data>(&data).unwrap();
    for bot in json.bots {
        manager.add_bot(bot.username, bot.password, bot.token, bot.login_method);
    }

    webserver::start(Arc::new(manager));
}
