use std::collections::HashMap;

use crate::types::{
    bot_info::{Info, Server, State},
    elogin_method::ELoginMethod,
    login_info::LoginInfo,
    vector::Vector2,
};

struct Bot {
    info: Info,
    state: State,
    server: Server,
    position: Vector2,
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        recovery_code: String,
        login_method: ELoginMethod,
    ) -> Self {
        Self {
            info: Info {
                username,
                password,
                recovery_code,
                login_method,
                login_info: LoginInfo::new(),
                ..Default::default()
            },
            state: State::default(),
            server: Server::default(),
            position: Vector2::default(),
        }
    }

    pub fn to_http(&mut self) {
        let req = ureq::post("https://www.growtopia1.com/growtopia/server_data.php").set(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        );

        let res = req.send_string("").unwrap();
        let body = res.into_string().unwrap();
        self.parse_server_data(body);
    }

    pub fn parse_server_data(&mut self, data: String) {
        self.info.server_data = data
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '|');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                    _ => None,
                }
            })
            .collect::<HashMap<String, String>>();
    }
}
