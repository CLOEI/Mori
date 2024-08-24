use std::{
    fs::{self, File},
    io::{Read, Write},
};

use crate::types::config::{BotConfig, Config};

pub fn parse_config() -> Result<Config, ()> {
    if fs::metadata("config.json").is_ok() {
        let mut f = File::open("config.json").unwrap();
        let mut contents = String::new();
        f.read_to_string(&mut contents).unwrap();
        let j: Config = serde_json::from_str(&contents).unwrap();
        return Ok(j);
    }
    Err(())
}

pub fn add_bot(bot: BotConfig) {
    let mut config = parse_config().unwrap();
    config.bots.push(bot);
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_bots() -> Vec<BotConfig> {
    let config = parse_config().unwrap();
    config.bots
}

pub fn remove_bot(username: String) {
    let mut config = parse_config().unwrap();
    config.bots.retain(|x| x.username != username);
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn edit_timeout(timeout: u32) {
    let mut config = parse_config().unwrap();
    config.timeout = timeout;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_timeout() -> u32 {
    let config = parse_config().unwrap();
    config.timeout
}

pub fn edit_findpath_delay(findpath_delay: u32) {
    let mut config = parse_config().unwrap();
    config.findpath_delay = findpath_delay;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn save_token_to_bot(username: String, token: String, data: String) {
    let mut config = parse_config().unwrap();
    for bot in config.bots.iter_mut() {
        if bot.username == username {
            bot.token = token.clone();
            bot.data = data.clone();
        }
    }
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_findpath_delay() -> u32 {
    let config = parse_config().unwrap();
    config.findpath_delay
}

pub fn get_selected_bot() -> String {
    let config = parse_config().unwrap();
    config.selected_bot
}

pub fn set_selected_bot(username: String) {
    let mut config = parse_config().unwrap();
    config.selected_bot = username;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}
