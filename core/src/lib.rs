use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::{Mutex, RwLock};
use crate::types::bot::{Info, State};
use crate::types::login_info::LoginInfo;

mod server;
mod types;
mod utils;
mod login;

pub struct Bot {
    pub host: Mutex<rusty_enet::Host<UdpSocket>>,
    pub peer_id: Mutex<Option<rusty_enet::PeerID>>,
    pub info: Info,
    pub state: State,
    pub position: RwLock<(f32, f32)>,
    pub logs: RwLock<Vec<String>>,
}

impl Bot {
    pub fn new(payload: Vec<String>) -> Self {
        let socket = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))).unwrap();
        let host = rusty_enet::Host::<UdpSocket>::new(
            socket,
            rusty_enet::HostSettings {
                peer_limit: 1,
                channel_limit: 2,
                compressor: Some(Box::new(rusty_enet::RangeCoder::new())),
                checksum: Some(Box::new(rusty_enet::crc32)),
                using_new_packet: true,
                ..Default::default()
            },
        ).expect("Failed to create host");

        Self {
            host: Mutex::new(host),
            peer_id: Mutex::new(None),
            info: Info {
                payload,
                login_method: types::bot::ELoginMethod::default(),
                token: Mutex::new(None),
                login_info: Mutex::new(None),
                server_data: Mutex::new(None),
                dashboard_links: Mutex::new(None),
            },
            position: RwLock::new((0.0, 0.0)),
            state: State {
                is_running: Mutex::new(false),
                is_redirecting: Mutex::new(false),
            },
            logs: RwLock::new(Vec::new()),
        }
    }

    pub fn logon(&self, data: Option<&str>) {
        if data.is_some() {
            todo!("Implement logon with pre-existing credentials");
        } else {
            let mut info = self.info.login_info.lock().unwrap();
            *info = Some(LoginInfo::new());
        }
        self.process_event();
    }

    pub fn connect_to_server(&self) {
        if *self.state.is_redirecting.lock().unwrap() {
            todo!("Handle redirection logic");
        } else {
            {
                let mut login_info = self.info.login_info.lock().unwrap();
                let info_data = login_info.as_mut().expect("Login info not set");
                let server_data = server::get_server_data(false, info_data);
                match server_data {
                    Ok(data) => {
                        info_data.meta = data.meta.clone();
                        let mut server = self.info.server_data.lock().unwrap();
                        *server = Some(data.clone());
                        let dashboard_data = server::get_dashboard(&data.loginurl, info_data).expect("Failed to get dashboard data");
                        let mut dashboard = self.info.dashboard_links.lock().unwrap();
                        *dashboard = Some(dashboard_data);
                    }
                    Err(e) => {
                        todo!("Handle error: {}", e);
                    }
                }
            }
            self.get_token();
        }

        let server_data = self.info.server_data.lock().unwrap();
        let server = server_data.as_ref().expect("Server data not set");

        let socket_address = SocketAddr::from_str(&format!("{}:{}", server.server, server.port)).unwrap();

        let mut host = self.host.lock().unwrap();
        match host.connect(socket_address, 2, 0) {
            Err(err) => {
                panic!("Failed to connect to server: {}", err);
            }
            _ => {}
        }
    }

    pub fn get_token(&self) {
        let dashboard_links = self.info.dashboard_links.lock().unwrap();
        let urls = dashboard_links.as_ref();
        let token = match self.info.login_method {
            types::bot::ELoginMethod::LEGACY => {
                let payload = self.info.payload.clone();
                let username = payload.get(0).expect("Username not found");
                let password = payload.get(1).expect("Password not found");
                let growtopia_url = urls.as_ref().and_then(|links| links.growtopia.clone());
                login::get_legacy_token(&growtopia_url.unwrap(), username, password).expect("Failed to get legacy token")
            },
            _ => {
                todo!("Implement token retrieval for different login methods");
            }
        };

        let mut token_lock = self.info.token.lock().unwrap();
        *token_lock = Some(token);
    }

    fn process_event(&self) {
        self.connect_to_server();

        loop {
            let event = {
                let mut host = self.host.lock().unwrap();
                let event = host.service();
                event.ok().flatten().map(|e| e.no_ref())
            };

            if let Some(event) = event {
                match event {
                    rusty_enet::EventNoRef::Connect { peer, .. } => {
                        println!("Connected to server");
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bot_logon() {
        let bot = Bot::new(vec!["".to_string(), "".to_string()]);
        bot.logon(None);
    }
}