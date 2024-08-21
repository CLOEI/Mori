use std::collections::HashMap;

use super::{elogin_method::ELoginMethod, login_info::LoginInfo};

#[derive(Debug, Default)]
pub struct Info {
    pub username: String,
    pub password: String,
    pub recovery_code: String,
    pub login_method: ELoginMethod,
    pub oauth_links: HashMap<String, String>,
    pub server_data: HashMap<String, String>,
    pub token: String,
    pub login_info: LoginInfo,
    pub ping: u32,
    pub status: String,
    pub timeout: u32,
}

#[derive(Debug, Default)]
pub struct State {
    pub net_id: u32,
    pub gems: i32,
    pub is_running: bool,
    pub is_redirecting: bool,
    pub is_ingame: bool,
    pub is_not_allowed_to_warp: bool,
    pub is_banned: bool,
}

#[derive(Debug, Default)]
pub struct Server {
    pub ip: String,
    pub port: u16,
}
