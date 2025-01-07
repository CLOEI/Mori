use std::{
    fs::{self, File},
    io::{Read, Write},
};

use crate::types::config::{BotConfig, Config};
use crate::utils;

use super::captcha::CaptchaProvider;

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
    config.bots.retain(|x| {
        let payload = utils::textparse::parse_and_store_as_vec(&x.payload);
        payload[0] != username
    });
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn set_timeout(timeout: u32) {
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

pub fn set_findpath_delay(findpath_delay: u32) {
    let mut config = parse_config().unwrap();
    config.findpath_delay = findpath_delay;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn save_token_to_bot(username: String, token: String, data: String) {
    let mut config = parse_config().unwrap();
    for bot in config.bots.iter_mut() {
        let payload = utils::textparse::parse_and_store_as_vec(&bot.payload);
        if payload[0] == username {
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

pub fn get_game_version() -> String {
    let config = parse_config().unwrap();
    config.game_version
}

pub fn set_game_version(version: String) {
    let mut config = parse_config().unwrap();
    config.game_version = version;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_proxies() -> Vec<crate::types::config::Proxy> {
    let config = parse_config().unwrap();
    config.proxy
}

pub fn add_proxy(proxy: crate::types::config::Proxy) {
    let mut config = parse_config().unwrap();
    config.proxy.push(proxy);
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn remove_proxy(index: usize) {
    let mut config = parse_config().unwrap();
    config.proxy.remove(index);
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_use_alternate_server() -> bool {
    let config = parse_config().unwrap();
    config.use_alternate_server
}

pub fn set_use_alternate_server(use_alternate_server: bool) {
    let mut config = parse_config().unwrap();
    config.use_alternate_server = use_alternate_server;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_bot_use_proxy(username: String) -> bool {
    let config = parse_config().unwrap();
    for b in config.bots.iter() {
        let payload = utils::textparse::parse_and_store_as_vec(&b.payload);
        if payload[0] == username {
            return b.use_proxy;
        }
    }
    false
}

pub fn set_auto_collect(auto_collect: bool) {
    let mut config = parse_config().unwrap();
    config.auto_collect = auto_collect;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn get_auto_collect() -> bool {
    let config = parse_config().unwrap();
    config.auto_collect
}

pub fn get_captcha_provider() -> CaptchaProvider {
    let config = parse_config().unwrap();
    config.captcha.provider
}

pub fn get_captcha_api_key() -> String {
    let config = parse_config().unwrap();
    config.captcha.api_key
}

pub fn set_captcha_provider(provider: CaptchaProvider) {
    let mut config = parse_config().unwrap();
    config.captcha.provider = provider;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}

pub fn set_captcha_api_key(api_key: String) {
    let mut config = parse_config().unwrap();
    config.captcha.api_key = api_key;
    let j = serde_json::to_string_pretty(&config).unwrap();
    let mut file = File::create("config.json").unwrap();
    file.write_all(j.as_bytes()).unwrap();
}
