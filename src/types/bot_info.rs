use std::collections::HashMap;

use super::{e_login_method::ELoginMethod, login_info::LoginInfo};

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
    pub status: String,
}

impl Default for Info {
    fn default() -> Self {
        Info {
            display_name: String::new(),
            username: String::new(),
            password: String::new(),
            code: "None".to_string(),
            method: ELoginMethod::LEGACY,
            oauth_links: Vec::new(),
            token: "None".to_string(),
            login_info: LoginInfo::default(),
            ping: 0,
            parsed_server_data: HashMap::new(),
            status: String::new(),
        }
    }
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

pub struct Server {
    pub ip: String,
    pub port: String,
}

impl Default for Server {
    fn default() -> Self {
        Server {
            ip: "0.0.0.0".to_string(),
            port: "00000".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
