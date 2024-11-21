use std::collections::HashMap;

use super::{elogin_method::ELoginMethod, login_info::LoginInfo};

#[derive(Debug, Default)]
pub struct Info {
    pub payload: Vec<String>,
    pub recovery_code: String,
    pub login_method: ELoginMethod,
    pub oauth_links: Vec<String>,
    pub server_data: HashMap<String, String>,
    pub token: String,
    pub login_info: LoginInfo,
    pub status: String,
    pub proxy: Option<ProxyInfo>,
}

#[derive(Debug)]
pub struct ProxyInfo {
    pub username: String,
    pub password: String,
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Default)]
pub struct State {
    pub net_id: u32,
    pub level: i32,
    pub playtime: u32,
    pub install_date: u32,
    pub gems: i32,
    pub is_running: bool,
    pub is_redirecting: bool,
    pub is_ingame: bool,
    pub is_not_allowed_to_warp: bool,
    pub is_banned: bool,
    pub is_tutorial: bool,
}

#[derive(Debug, Default)]
pub struct Server {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Default, Clone)]
pub struct FTUE {
    // First Time User Experience
    pub current_progress: i32,
    pub total_progress: i32,
    pub info: String,
}

#[derive(Debug, Default)]
pub struct TemporaryData {
    pub drop: (u32, u32),
    pub trash: (u32, u32),
    pub timeout: u32,
    pub ping: u32,
    pub entered_world: bool,
}
