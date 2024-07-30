use std::collections::HashMap;

use super::{e_login_method::ELoginMethod, login_info::LoginInfo};

#[derive(Default)]
pub struct Info {
    pub display_name: String,
    pub username: String,
    pub password: String,
    pub code: String,
    pub method: ELoginMethod,
    pub oauth_links: Vec<String>,
    pub token: String,
    pub login_info: LoginInfo,
    pub ping: u32,
    pub parsed_server_data: HashMap<String, String>,
}

#[derive(Default)]
pub struct State {
    pub net_id: u32,
    pub gems: i32,
    pub is_redirect: bool,
    pub is_running: bool,
    pub is_banned: bool,
    pub is_ingame: bool,
}

#[derive(Default)]
pub struct Server {
    pub ip: String,
    pub port: String,
}

#[derive(Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
