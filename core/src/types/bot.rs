use std::sync::Mutex;
use crate::server::DashboardLinks;
use crate::types::login_info::LoginInfo;
use crate::types::server_data::ServerData;

pub struct State {
    pub is_running: Mutex<bool>,
    pub is_redirecting: Mutex<bool>,
}

pub struct Info {
    pub payload: Vec<String>,
    pub login_method: ELoginMethod,
    pub login_info: Mutex<Option<LoginInfo>>,
    pub server_data: Mutex<Option<ServerData>>,
    pub dashboard_links: Mutex<Option<DashboardLinks>>,
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