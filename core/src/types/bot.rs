use std::sync::Mutex;
use std::time::Duration;
use crate::server::DashboardLinks;
use crate::types::login_info::LoginInfo;
use crate::types::player::Player;
use crate::types::server_data::ServerData;

#[derive(Debug, Default)]
pub struct State {
    pub hack_type: u32,
    pub build_length: u8,
    pub punch_length: u8,
    pub velocity: f32,
    pub gravity: f32,
}

pub struct Info {
    pub payload: Vec<String>,
    pub login_method: ELoginMethod,
    pub login_info: Mutex<Option<LoginInfo>>,
    pub server_data: Mutex<Option<ServerData>>,
    pub dashboard_links: Mutex<Option<DashboardLinks>>,
}

#[derive(Debug, Default)]
pub struct World {
    pub data: Mutex<Option<gtworld_r::World>>,
    pub players: Mutex<Vec<Player>>
}

#[derive(Debug, PartialEq, Clone)]
pub enum ELoginMethod {
    APPLE,
    GOOGLE,
    LEGACY,
}

impl Default for ELoginMethod {
    fn default() -> Self {
        ELoginMethod::LEGACY
    }
}