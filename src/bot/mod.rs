use spdlog::info;

use crate::connect::Connect;
use crate::types::e_login_method::ELoginMethod;

pub struct Bot {
    username: String,
    password: String,
    method: ELoginMethod,
    oauth_links: Vec<String>,
    is_running: bool,
    token: String,
    connect: Connect,
}

impl Bot {
    pub fn new(
        username: String,
        password: String,
        method: ELoginMethod,
        oauth_links: Vec<String>,
    ) -> Bot {
        Bot {
            username,
            password,
            method,
            oauth_links,
            is_running: false,
            token: String::new(),
            connect: Connect::new(),
        }
    }
}

impl Bot {
    pub fn start(&mut self) {
        info!("Getting token for {}", self.username);
        match self.method {
            ELoginMethod::APPLE => {
                self.connect.get_apple_token(
                    self.oauth_links[0].as_str(),
                    self.username.as_str(),
                    self.password.as_str(),
                );
            }
            ELoginMethod::GOOGLE => {
                self.connect.get_google_token(
                    self.oauth_links[1].as_str(),
                    self.username.as_str(),
                    self.password.as_str(),
                );
            }
            ELoginMethod::LEGACY => {
                let res = self
                    .connect
                    .get_legacy_token(
                        self.oauth_links[2].as_str(),
                        self.username.as_str(),
                        self.password.as_str(),
                    )
                    .unwrap();
                self.token = res;
            }
        }
        info!("Received the token: {}", self.token);
    }

    fn stop(&mut self) {
        self.is_running = false;
    }
}
