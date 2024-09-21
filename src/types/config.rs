use serde::{Deserialize, Serialize};

use super::elogin_method::ELoginMethod;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bots: Vec<BotConfig>,
    pub proxy: Vec<Proxy>,
    pub timeout: u32,
    pub findpath_delay: u32,
    pub selected_bot: String,
    pub game_version: String,
    pub use_alternate_server: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotConfig {
    pub payload: String,
    pub recovery_code: String,
    pub login_method: ELoginMethod,
    pub token: String,
    pub data: String,
    pub use_proxy: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proxy {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}