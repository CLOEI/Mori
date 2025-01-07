use super::{elogin_method::ELoginMethod, login_info::LoginInfo};
use crate::core::Bot;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Info {
    pub payload: Vec<String>,
    pub recovery_code: String,
    pub login_method: ELoginMethod,
    pub oauth_links: Vec<String>,
    pub server_data: HashMap<String, String>,
    pub token: String,
    pub login_info: LoginInfo,
    pub status: EStatus,
    pub proxy: Option<ProxyInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EStatus {
    Unauthorized,
    TwoFAFailed,
    ParsingServer,
    FetchingServer,
    CheckingToken,
    GettingToken,
    GettingOAuth,
    UpdatingData,
    SpoofingData,
    Disconnected,
    Reconnecting,
    Connecting,
    Connected,
    Banned
}

impl Default for EStatus {
    fn default() -> Self {
        EStatus::Disconnected
    }
}

impl EStatus {
    pub fn to_string(&self) -> String {
        match self {
            EStatus::Unauthorized => "Unauthorized".to_string(),
            EStatus::TwoFAFailed => "2FA Failed".to_string(),
            EStatus::ParsingServer => "Parsing server".to_string(),
            EStatus::FetchingServer => "Fetching server".to_string(),
            EStatus::CheckingToken => "Checking token".to_string(),
            EStatus::GettingToken => "Getting token".to_string(),
            EStatus::GettingOAuth => "Getting OAuth".to_string(),
            EStatus::UpdatingData => "Updating data".to_string(),
            EStatus::SpoofingData => "Spoofing data".to_string(),
            EStatus::Disconnected => "Disconnected".to_string(),
            EStatus::Reconnecting => "Reconnecting".to_string(),
            EStatus::Connecting => "Connecting".to_string(),
            EStatus::Connected => "Connected".to_string(),
            EStatus::Banned => "Banned".to_string(),
        }
    }
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
    pub is_allowed_to_warp: bool,
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
    pub dialog_callback: Option<fn(&Arc<Bot>)>,
}
