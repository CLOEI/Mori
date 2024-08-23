use serde::{Deserialize, Serialize};

use super::elogin_method::ELoginMethod;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bots: Vec<BotConfig>,
    pub timeout: u32,
    pub findpath_delay: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub username: String,
    pub password: String,
    pub recovery_code: String,
    pub login_method: ELoginMethod,
    pub token: String,
    pub data: String,
}
